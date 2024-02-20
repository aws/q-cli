pub trait IntoMetricDatum: Send {
    fn into_metric_datum(self, credential_start_url: Option<String>) -> ::amzn_toolkit_telemetry::types::MetricDatum;
}

include!(concat!(env!("OUT_DIR"), "/mod.rs"));
