use std::collections::HashSet;
use std::time::Instant;

use opentelemetry_api::KeyValue;
use opentelemetry_api::Value;
use tracing_core::field::Visit;
use tracing_core::span;
use tracing_core::Field;
use tracing_core::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use crate::plugins::telemetry::consts::EXECUTION_SPAN_NAME;
use crate::plugins::telemetry::consts::QUERY_PLANNING_SPAN_NAME;
use crate::plugins::telemetry::consts::REQUEST_SPAN_NAME;
use crate::plugins::telemetry::consts::SUBGRAPH_SPAN_NAME;
use crate::plugins::telemetry::consts::SUPERGRAPH_SPAN_NAME;

const SUBGRAPH_ATTRIBUTE_NAME: &str = "apollo.subgraph.name";

#[derive(Debug)]
pub(crate) struct SpanMetricsLayer {
    span_names: HashSet<&'static str>,
}

impl Default for SpanMetricsLayer {
    fn default() -> Self {
        Self {
            span_names: [
                REQUEST_SPAN_NAME,
                SUPERGRAPH_SPAN_NAME,
                SUBGRAPH_SPAN_NAME,
                QUERY_PLANNING_SPAN_NAME,
                EXECUTION_SPAN_NAME,
            ]
            .into(),
        }
    }
}

impl<S> Layer<S> for SpanMetricsLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();

        let name = attrs.metadata().name();
        if self.span_names.contains(name) && extensions.get_mut::<Timings>().is_none() {
            let mut timings = Timings::new();
            if name == SUBGRAPH_SPAN_NAME {
                attrs.values().record(&mut ValueVisitor {
                    timings: &mut timings,
                });
            }
            extensions.insert(Timings::new());
        }
    }

    fn on_record(&self, _span: &span::Id, _values: &span::Record<'_>, _ctx: Context<'_, S>) {}

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();

        if let Some(timings) = extensions.get_mut::<Timings>() {
            let duration = timings.start.elapsed().as_secs_f64();

            // JASON customization: add lables azure_region to apollo_router_span
            let azure_region = crate::uhg_custom::get_uhg_azure_region();

            // Convert it in seconds
            let idle: f64 = timings.idle as f64 / 1_000_000_000_f64;
            let busy: f64 = timings.busy as f64 / 1_000_000_000_f64;
            let name = span.metadata().name();

            // JASON customization - begin!
            if let Some(subgraph_name) = timings.subgraph.take() {
                record(duration, "duration", name, Some(&subgraph_name), &azure_region);
                record(idle, "idle", name, Some(&subgraph_name), &azure_region);
                record(busy, "busy", name, Some(&subgraph_name), &azure_region);
            } else {
                record(duration, "duration", name, None, &azure_region);
                record(idle, "idle", name, None, &azure_region);
                record(busy, "busy", name, None, &azure_region);
            }
            // JASON customization - end!
        }
    }

    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();

        if let Some(timings) = extensions.get_mut::<Timings>() {
            let now = Instant::now();
            timings.idle += (now - timings.last).as_nanos() as i64;
            timings.last = now;
        }
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();

        if let Some(timings) = extensions.get_mut::<Timings>() {
            let now = Instant::now();
            timings.busy += (now - timings.last).as_nanos() as i64;
            timings.last = now;
        }
    }
}

fn record(duration: f64, kind: &'static str, name: &str, subgraph_name: Option<&str>, azure_region: &str) {
    // Avoid a heap allocation for a vec by using a slice
    let attrs = [
        KeyValue::new("kind", kind),
        KeyValue::new("span", Value::String(name.to_string().into())),
        KeyValue::new(
            "subgraph",
            Value::String(
                subgraph_name
                    .map(|s| s.to_string().into())
                    .unwrap_or_else(|| "".into()),
            ),
        ),

        // JASON customization: add lables azure_region
        KeyValue::new("azure_region", Value::String(azure_region.to_string().into())),
    ];
    let splice = if subgraph_name.is_some() {
        &attrs
    } else {
        &attrs[0..2]
    };

    f64_histogram!("apollo_router_span", "Duration of span", duration, splice);
}

struct Timings {
    idle: i64,
    busy: i64,
    last: Instant,
    start: Instant,
    subgraph: Option<String>,
}

impl Timings {
    fn new() -> Self {
        Self {
            idle: 0,
            busy: 0,
            last: Instant::now(),
            start: Instant::now(),
            subgraph: None,
        }
    }
}

struct ValueVisitor<'a> {
    timings: &'a mut Timings,
}

impl<'a> Visit for ValueVisitor<'a> {
    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == SUBGRAPH_ATTRIBUTE_NAME {
            self.timings.subgraph = Some(value.to_string());
        }
    }
}
