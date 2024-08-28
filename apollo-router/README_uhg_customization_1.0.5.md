# branch
- dev2
- dev-hcp ------ this is the branch to create cra

C:\GitHub\router\apollo-router\Cargo.toml
[package]
name = "uhg-custom-appollo-roouter"
version = "1.0.5"

C:\GitHub\router\apollo-router-benchmarks\Cargo.toml
uhg-custom-appollo-roouter = { path = "../apollo-router" }

basically replace all "apollo-router" with "uhg-custom-appollo-roouter"

1. C:\GitHub\router\apollo-router\src\axum_factory\utils.rs

impl PropagatingMakeSpan {
    fn create_span<B>(&mut self, request: &Request<B>) -> Span {
        // JASON customization - add labels: consumerName, roles, correlationId, cid, azure_region
        let consumer_name = match request.headers().get("consumername") {
            Some(s) => std::str::from_utf8(s.as_bytes()).unwrap(),
            None => ""
        };
        let role_id = match request.headers().get("roleid") {
            Some(s) => std::str::from_utf8(s.as_bytes()).unwrap(),
            None => ""
        };
        let id = String::from("");
        let correlation_id = match request.headers().get("X-Correlation-Id") {
            Some(s) => std::str::from_utf8(s.as_bytes()).unwrap(),
            None => {
                // can't get trace id here! leave it empty, will be overriden in plugin repo
                // match crate::tracer::TraceId::maybe_new().map(|t| t.to_string()) {
                //     Some(v) => { id = format!("hcp-{}", v); }
                //     None => { }
                // };

                &id
            }
        };        
        let cid = match request.headers().get("Optum-Cid-Ext") {
            Some(s) => std::str::from_utf8(s.as_bytes()).unwrap(),
            None => ""
        };
        let azure_region = std::env::var("AZURE_REGION").unwrap_or(String::from(""));
        // end of JASON customization
        
       

        tracing::error_span!(
                REQUEST_SPAN_NAME,
                "consumerName" = consumer_name,
                "roles" = role_id,
                "correlationId" = correlation_id,
                "cid" = cid,
                "azure_region" = azure_region,
                "http.method" = %request.method(),
                "http.route" = %request.uri(),
                "http.flavor" = ?request.version(),
                "http.status" = 500, // This prevents setting later
                "otel.name" = ::tracing::field::Empty,
                "otel.kind" = "SERVER",
                "graphql.operation.name" = ::tracing::field::Empty,
                "graphql.operation.type" = ::tracing::field::Empty,
                "apollo_router.license" = LICENSE_EXPIRED_SHORT_MESSAGE,
                "apollo_private.request" = true,
            )

        tracing::info_span!(
                REQUEST_SPAN_NAME,
                "consumerName" = consumer_name,
                "roles" = role_id,
                "correlationId" = correlation_id,
                "cid" = cid,
                "azure_region" = azure_region,
                "http.method" = %request.method(),
                "http.route" = %request.uri(),
                "http.flavor" = ?request.version(),
                "otel.name" = ::tracing::field::Empty,
                "otel.kind" = "SERVER",
                "graphql.operation.name" = ::tracing::field::Empty,
                "graphql.operation.type" = ::tracing::field::Empty,
                "apollo_private.request" = true,
            )

2. C:\GitHub\router\apollo-router\src\plugins\telemetry\metrics\prometheus.rs
    
    use crate::plugins::telemetry::metrics::custom_aggregator::CustomHistogramAggregator;

impl MetricsConfigurator for Config {
    fn apply(
        &self,
        mut builder: MetricsBuilder,
        metrics_config: &MetricsCommon,
    ) -> Result<MetricsBuilder, BoxError> {
        if self.enabled {
            let mut controller = controllers::basic(
                processors::factory(
                    // JASON customization - CustomHistogramAggregator
                    CustomHistogramAggregator::new(metrics_config),
                    aggregation::stateless_temporality_selector(),
                )
                .with_memory(true),
            )
            .with_resource(Resource::new(
                metrics_config
                    .resources
                    .clone()
                    .into_iter()
                    .map(|(k, v)| KeyValue::new(k, v)),
            ))
            .build();

            // Check the last controller to see if the resources are the same, if they are we can use it as is.
            // Otherwise go with the new controller and store it so that it can be committed during telemetry activation.
            if let Some(last_controller) = CONTROLLER.lock().expect("lock poisoned").clone() {
                if controller.resource() == last_controller.resource() {
                    tracing::debug!("prometheus controller can be reused");
                    controller = last_controller
                } else {
                    tracing::debug!("prometheus controller cannot be reused");
                }
            }
            NEW_CONTROLLER
                .lock()
                .expect("lock poisoned")
                .replace(controller.clone());

            let exporter = opentelemetry_prometheus::exporter(controller).try_init()?;

            builder = builder.with_custom_endpoint(
                self.listen.clone(),
                Endpoint::from_router_service(
                    self.path.clone(),
                    PrometheusService {
                        registry: exporter.registry().clone(),
                    }
                    .boxed(),
                ),
            );
            builder = builder.with_meter_provider(exporter.meter_provider()?);
            builder = builder.with_exporter(exporter);
            tracing::info!(
                "Prometheus endpoint exposed at {}{}",
                self.listen,
                self.path
            );
        }
        Ok(builder)
    }
}


3. new file:  C:\GitHub\router\apollo-router\src\plugins\telemetry\metrics\custom_aggregator.rs
use std::collections::HashMap;
use std::sync::Arc;

use opentelemetry::sdk::export::metrics::AggregatorSelector;
use opentelemetry::sdk::metrics::aggregators;
use opentelemetry::sdk::metrics::aggregators::Aggregator;
use opentelemetry::sdk::metrics::sdk_api::Descriptor;
use opentelemetry::sdk::metrics::sdk_api::InstrumentKind;
use crate::plugins::telemetry::config::MetricsCommon;

// JASON customization - add labels: consumerName, correlationId, cid

//
// Metrics CustomAggregator
// reference: https://github.com/open-telemetry/opentelemetry-rust/blob/bfeda30583f3df6733e8ebce2f5964f6a7b69b5c/examples/dynatrace/src/main.rs
//

#[derive(Debug, Clone)]
pub(crate) struct CustomHistogramAggregator {
    buckets: Option<HashMap<String, Vec<f64>>>,
    default_bucket: Vec<f64>
}

impl CustomHistogramAggregator {
    pub(crate) fn new(metrics_config: &MetricsCommon) -> CustomHistogramAggregator {
        let mut buckets: Option<HashMap<String, Vec<f64>>> = None;
        let mut default_bucket: Vec<f64> = vec![0.001, 0.005, 0.015, 0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 1.0, 5.0, 10.0];

        if metrics_config.buckets.is_some() {
            let buckets_all = metrics_config.buckets.as_ref().unwrap().clone();

            if buckets_all.contains_key("default") {
                default_bucket = buckets_all.get("default").unwrap().clone();
            }

            buckets = Some(buckets_all);
        }

        CustomHistogramAggregator { 
            buckets, 
            default_bucket 
        }
    }
}

impl AggregatorSelector for CustomHistogramAggregator {
    fn aggregator_for(
        &self,
        descriptor: &Descriptor,
    ) -> Option<Arc<(dyn Aggregator + Sync + std::marker::Send + 'static)>> {
        match descriptor.instrument_kind() {
            // Histogram
            InstrumentKind::Histogram => {
                let boundaries = match &self.buckets {
                    Some(buckets) => {
                        match buckets.get(descriptor.name()) {
                            Some(found_buckets) => found_buckets,
                            None => &self.default_bucket
                        }
                    },
                    None => &self.default_bucket
                };

                Some(Arc::new(aggregators::histogram(boundaries)))
            },

            // Gauge
            InstrumentKind::GaugeObserver => Some(Arc::new(aggregators::last_value())),

            // InstrumentKind::Counter, UpDownCounter, CounterObserver, UpDownCounterObserver
            _ => Some(Arc::new(aggregators::sum()))
        }
    }
}

4. unit tests
    apollo-router/tests/snapshots/tracing_tests__traced_basic_composition.snap

    "entries": [
          [
            "consumerName",
            ""
          ],
          [
            "roles",
            ""
          ],
          [
            "correlationId",
            ""
          ],
          [
            "cid",
            ""
          ],
          [
            "azure_region",
            ""
          ],
          [
            "http.method",

    apollo-router/tests/snapshots/tracing_tests__traced_basic_request.snap

    "fields": {
            "names": [
              "consumerName",
              "roles",
              "correlationId",
              "cid",
              "azure_region",
              "http.method",

    apollo-router/tests/snapshots/tracing_tests__variables.snap
        contains changes shown in above 2 files