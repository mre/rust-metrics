use metrics::counter::StdCounter;
use metrics::gauge::StdGauge;
use metrics::meter::MeterSnapshot;
/// a Metric
use histogram::Histogram;

pub trait Snaphot {
    fn export_metric(&self) -> MetricValue;
}

impl Snapshot for Histogram {
    fn export_metric(&self) -> MetricValue {
        MetricValue::Histogram(self.clone())
    }
}

pub enum MetricValue {
    Counter(StdCounter),
    Gauge(StdGauge),
    Meter(MeterSnapshot),
    Histogram(Histogram),
}
