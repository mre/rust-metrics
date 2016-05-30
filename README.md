# rust-metrics
[![Linux Status](https://travis-ci.org/posix4e/rust-metrics.svg?branch=master)](https://travis-ci.org/posix4e/rust-metrics)

Metrics collection for Rust. You can easily shift between telemetry
aggregation solutions as code without changing the code to write or
read metrics. In theory it could be the basis of any number of metrics
libraries.
## Usage

Add this to your `Cargo.toml`:

```toml
"metrics" = "0.1.1"
```

And add this to your crate root:

```rust
extern crate metrics
```

## Features

- [x] Gauges
- [x] Counters
- [x] Meters
- [x] Console Based Reporter
- [x] Create a more basic histogram trait and MetricValue
- [x] Histogram support
- [x] Graphite reporter
- [ ] C collector
- [ ] JS collector
- [ ] High overhead/Low resolution Collectors (http, statsd, shared memory, fs)
- [In progress] Prometheus support
- [ ] Only link in what is need in production
- [ ] A better summary object
- [ ] Tested in Production

## License

`rust-metrics` is primarily distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.

Copyright (c) 2015 Alex Newman.
