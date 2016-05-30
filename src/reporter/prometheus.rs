use metrics::metric::Metric;
use registry::{Registry, StdRegistry};
use std::time::Duration;
use std::thread;
use std::sync::Arc;
use metrics::meter::Meter;
use reporter::base::Reporter;
use metrics::counter::StdCounter;
use metrics::gauge::StdGauge;
use metrics::meter::MeterSnapshot;
use histogram::Histogram;
use time;
use time::Timespec;

use router::{Router, NoRoute};
use std::result;
use iron;
use iron::typemap::Key;
use iron::prelude::*;
use iron::status;

use promo_proto;
use persistent::Read;
use protobuf::Message;
use promo_proto::MetricFamily;

#[derive(Copy, Clone)]
struct foo;

pub struct PrometheusReporter {
    host_and_port: &'static str,
    prefix: &'static str,
    registry: Arc<StdRegistry<'static>>,
    reporter_name: &'static str,
}

impl Key for foo { type Value = Arc<StdRegistry<'static>>; }

impl Reporter for PrometheusReporter {
    fn get_unique_reporter_name(&self) -> &'static str {
        self.reporter_name
    }
}

fn prefix(metric_line: String, prefix_str: &'static str) -> String {
    format!("{}.{}", prefix_str, metric_line)
}

impl PrometheusReporter {
    pub fn new(registry: Arc<StdRegistry<'static>>,
               reporter_name: &'static str,
               host_and_port: &'static str,
               prefix: &'static str)
               -> PrometheusReporter {
        PrometheusReporter {
            host_and_port: host_and_port,
            prefix: prefix,
            registry: registry,
            reporter_name: reporter_name,
        }
    }

    pub fn start(self) -> thread::JoinHandle<iron::Listening> {
        thread::spawn(move || {
            let mut router = Router::new();
            router.get("/", handler);
            let mut chain = Chain::new(router);
            // The double long ARC pointer FTW!
            chain.link_before(Read::<foo>::one(self.registry));
            // TODO -> Result<iron::Listening, iron::error::Error>
            Iron::new(chain).http(self.host_and_port).unwrap()
        })
    }
}

fn handler(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, ser_to_pb(req.get::<Read<foo>>().unwrap()))))
}

fn ser_to_pb(registry: Arc<Arc<StdRegistry<'static>>>) -> Vec<u8> {
    let names = registry.get_metrics_names();
    MetricFamily::new().write_to_bytes().unwrap()
}

#[cfg(test)]
mod test {
    use metrics::meter::{Meter, StdMeter};
    use metrics::counter::{Counter, StdCounter};
    use metrics::gauge::{Gauge, StdGauge};
    use registry::{Registry, StdRegistry};
    use reporter::prometheus::PrometheusReporter;
    use std::sync::Arc;
    use histogram::*;

    #[test]
    fn add_some_stats_and_slurp_them_with_http() {
        extern crate hyper;
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
        let reporter =
            PrometheusReporter::new(arc_registry.clone(), "test", "0.0.0.0:8080", "asd.asdf");
        reporter.start();

        let client = hyper::client::Client::new();

        let res = client.get("http://127.0.0.1:8080").send().unwrap();


    }
}
