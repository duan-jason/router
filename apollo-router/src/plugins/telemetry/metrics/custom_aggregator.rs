use std::collections::HashMap;
use std::sync::Arc;

use opentelemetry::sdk::export::metrics::AggregatorSelector;
use opentelemetry::sdk::metrics::aggregators;
use opentelemetry::sdk::metrics::aggregators::Aggregator;
use opentelemetry::sdk::metrics::sdk_api::Descriptor;

use crate::plugins::telemetry::config::MetricsCommon;

//
// Metrics CustomAggregator
// reference: https://github.com/open-telemetry/opentelemetry-rust/blob/bfeda30583f3df6733e8ebce2f5964f6a7b69b5c/examples/dynatrace/src/main.rs
//

#[derive(Debug, Clone)]
pub(crate) struct CustomHistogramAggregator {
    buckets: HashMap<String, Vec<f64>>
}

impl CustomHistogramAggregator {
    pub(crate) fn new(metrics_config: &MetricsCommon) -> CustomHistogramAggregator {
        CustomHistogramAggregator {
            buckets: metrics_config.buckets.clone()
        }
    }
}

impl AggregatorSelector for CustomHistogramAggregator {
    fn aggregator_for(
        &self,
        descriptor: &Descriptor,
    ) -> Option<Arc<(dyn Aggregator + Sync + std::marker::Send + 'static)>> {
        match self.buckets.get(descriptor.name()) {
            Some(buckets) => histogram_buckets(buckets),
            None => histogram_buckets(&vec![0.001, 0.005, 0.015, 0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 1.0, 5.0, 10.0])
        }
    }
}

fn histogram_buckets(boundries: &Vec<f64>) -> Option<Arc<(dyn Aggregator + Sync + std::marker::Send + 'static)>> {
    Some(Arc::new(aggregators::histogram(boundries)))
}