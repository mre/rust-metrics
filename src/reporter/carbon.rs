// CarbonReporter sends a message to a carbon end point at a regular basis.
use registry::{Registry, StdRegistry};
use std::time::Duration;
use std::thread;
use std::sync::Arc;
use reporter::base::Reporter;
use metrics::counter::StdCounter;
use metrics::gauge::StdGauge;
use metrics::meter::MeterSnapshot;
use histogram::Histogram;
use time;
use time::Timespec;
use std::net::TcpStream;
use std::io::Write;

pub struct CarbonStream {
    graphite_stream: Option<TcpStream>,
    host_and_port: String,
}

pub struct CarbonReporter {
    host_and_port: String,
    prefix: &'static str,
    registry: Arc<StdRegistry<'static>>,
    reporter_name: &'static str,
}

impl CarbonStream {
    pub fn new(host_and_port: String) -> CarbonStream {
        CarbonStream {
            host_and_port: host_and_port,
            graphite_stream: None,
        }
    }

    pub fn connect(&mut self) {
        let host_and_port = &*self.host_and_port;
        match TcpStream::connect(host_and_port) {
            Ok(x) => self.graphite_stream = Some(x),
            Err(e) => panic!("Unable to connect to {} because {}", host_and_port, e),
        }

    }

    pub fn write(&mut self, metric_path: String, value: String, timespec: Timespec) {
        let seconds_in_ms = (timespec.sec * 1000) as u32;
        let nseconds_in_ms = (timespec.nsec / 1000) as u32;
        let timestamp = seconds_in_ms + nseconds_in_ms;
        match self.graphite_stream {
            Some(ref mut stream) => {
                let carbon_command = format!("{} {} {}\n", metric_path, value, timestamp)
                    .into_bytes();
                match stream.write_all(&carbon_command) {
                    Ok(_) => {}
                    Err(x) => println!("Failed to Send {:?}", x),
                }
            }
            None => {
                self.reconnect_stream();
                self.write(metric_path, value, timespec);
            }
        }
    }
    fn reconnect_stream(&mut self) {
        println!("Waiting 123ms and then reconnecting");
        thread::sleep(Duration::from_millis(123));
        self.connect();
    }
}

impl Reporter for CarbonReporter {
    fn get_unique_reporter_name(&self) -> &'static str {
        self.reporter_name
    }
}

fn prefix(metric_line: String, prefix_str: &'static str) -> String {
    format!("{}.{}", prefix_str, metric_line)
}

fn send_meter_metric(metric_name: String,
                     meter: MeterSnapshot,
                     carbon: &mut CarbonStream,
                     prefix_str: &'static str,
                     ts: Timespec) {

    let count = meter.count.to_string();
    let m1_rate = meter.rates[0].to_string();
    let m5_rate = meter.rates[1].to_string();
    let m15_rate = meter.rates[2].to_string();
    let mean_rate = meter.mean.to_string();
    carbon.write(prefix(format!("{}.count", metric_name), prefix_str),
                 count,
                 ts);
    carbon.write(prefix(format!("{}.m1", metric_name), prefix_str),
                 m1_rate,
                 ts);
    carbon.write(prefix(format!("{}.m5", metric_name), prefix_str),
                 m5_rate,
                 ts);
    carbon.write(prefix(format!("{}.m15", metric_name), prefix_str),
                 m15_rate,
                 ts);
    carbon.write(prefix(format!("{}.mean", metric_name), prefix_str),
                 mean_rate,
                 ts);
}

fn send_gauge_metric(metric_name: String,
                     gauge: StdGauge,
                     carbon: &mut CarbonStream,
                     prefix_str: &'static str,
                     ts: Timespec) {
    carbon.write(prefix(format!("{}", metric_name), prefix_str),
                 gauge.value.to_string(),
                 ts);
}

fn send_counter_metric(metric_name: String,
                       counter: StdCounter,
                       carbon: &mut CarbonStream,
                       prefix_str: &'static str,
                       ts: Timespec) {
    carbon.write(prefix(format!("{}", metric_name), prefix_str),
                 counter.value.to_string(),
                 ts);
}
fn send_histogram_metric(metric_name: String,
                         histogram: &mut Histogram,
                         carbon: &mut CarbonStream,
                         prefix_str: &'static str,
                         ts: Timespec) {
    let count = histogram.count();
    // let sum = histogram.sum();
    // let mean = sum / count;
    let max = histogram.percentile(100.0).unwrap();
    let min = histogram.percentile(0.0).unwrap();

    let p50 = histogram.percentile(50.0).unwrap();
    let p75 = histogram.percentile(75.0).unwrap();
    let p95 = histogram.percentile(95.0).unwrap();
    let p98 = histogram.percentile(98.0).unwrap();
    let p99 = histogram.percentile(99.0).unwrap();
    let p999 = histogram.percentile(99.9).unwrap();
    let p9999 = histogram.percentile(99.99).unwrap();
    let p99999 = histogram.percentile(99.999).unwrap();

    carbon.write(prefix(format!("{}.count", metric_name), prefix_str),
                 count.to_string(),
                 ts);

    carbon.write(prefix(format!("{}.max", metric_name), prefix_str),
                 max.to_string(),
                 ts);

    // carbon
    // .write(prefix(format!("{}.mean", metric_name), prefix_str),
    // mean.into_string(),
    // ts);

    carbon.write(prefix(format!("{}.min", metric_name), prefix_str),
                 min.to_string(),
                 ts);

    carbon.write(prefix(format!("{}.p50", metric_name), prefix_str),
                 p50.to_string(),
                 ts);

    carbon.write(prefix(format!("{}.p75", metric_name), prefix_str),
                 p75.to_string(),
                 ts);

    carbon.write(prefix(format!("{}.p95", metric_name), prefix_str),
                 p95.to_string(),
                 ts);

    carbon.write(prefix(format!("{}.p98", metric_name), prefix_str),
                 p98.to_string(),
                 ts);

    carbon.write(prefix(format!("{}.p99", metric_name), prefix_str),
                 p99.to_string(),
                 ts);

    carbon.write(prefix(format!("{}.p999", metric_name), prefix_str),
                 p999.to_string(),
                 ts);

    carbon.write(prefix(format!("{}.p9999", metric_name), prefix_str),
                 p9999.to_string(),
                 ts);

    carbon.write(prefix(format!("{}.p99999", metric_name), prefix_str),
                 p99999.to_string(),
                 ts);
}

impl CarbonReporter {
    pub fn new(registry: Arc<StdRegistry<'static>>,
               reporter_name: &'static str,
               host_and_port: String,
               prefix: &'static str)
               -> CarbonReporter {
        CarbonReporter {
            host_and_port: host_and_port,
            prefix: prefix,
            registry: registry,
            reporter_name: reporter_name,
        }
    }

    fn report_to_carbon_continuously(self, delay_ms: u32) -> thread::JoinHandle<()> {
        use metrics::metric::MetricValue::{Counter, Gauge, Histogram, Meter};

        let prefix = self.prefix;
        let host_and_port = self.host_and_port.clone();
        let mut carbon = CarbonStream::new(host_and_port);
        let registry = self.registry.clone();
        thread::spawn(move || {
            loop {
                let ts = time::now().to_timespec();
                for metric_name in &registry.get_metrics_names() {
                    let metric = registry.get(metric_name);
                    let mnas = metric_name.to_string(); // Metric name as string
                    match metric.export_metric() {
                        Meter(x) => send_meter_metric(mnas, x, &mut carbon, prefix, ts),
                        Gauge(x) => send_gauge_metric(mnas, x, &mut carbon, prefix, ts),
                        Counter(x) => send_counter_metric(mnas, x, &mut carbon, prefix, ts),
                        Histogram(mut x) => {
                            send_histogram_metric(mnas, &mut x, &mut carbon, prefix, ts)
                        }
                    }
                }
                thread::sleep(Duration::from_millis(delay_ms as u64));
            }
        })
    }

    pub fn start(self, delay_ms: u32) {
        self.report_to_carbon_continuously(delay_ms);
    }
}

#[cfg(test)]
mod test {
    use metrics::meter::{Meter, StdMeter};
    use metrics::counter::{Counter, StdCounter};
    use metrics::gauge::{Gauge, StdGauge};
    use registry::{Registry, StdRegistry};
    use reporter::carbon::CarbonReporter;
    use std::sync::Arc;
    use std::thread;
    use histogram::*;

    #[test]
    fn meter() {
        let m = StdMeter::new();
        m.mark(100);

        let mut c: StdCounter = StdCounter::new();
        c.inc();

        let mut g: StdGauge = StdGauge { value: 0f64 };
        g.set(1.2);

        let mut hc = HistogramConfig::new();
        hc.max_value(100).precision(1);
        let mut h = Histogram::configured(hc).unwrap();

        h.record(1, 1);

        let mut r = StdRegistry::new();
        r.insert("meter1", m);
        r.insert("counter1", c);
        r.insert("gauge1", g);
        r.insert("histogram", h);

        let arc_registry = Arc::new(r);
        CarbonReporter::new(arc_registry.clone(),
                            "test",
                            "localhost:0".to_string(),
                            "asd.asdf");
    }
}
