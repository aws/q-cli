pub trait IntoMetricDatum: Send {
    fn into_metric_datum(self) -> amzn_toolkit_telemetry::types::MetricDatum;
}

include!(concat!(env!("OUT_DIR"), "/mod.rs"));
