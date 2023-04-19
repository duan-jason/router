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