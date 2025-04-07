use std::time::{Duration, Instant};
use termitype::{config::Config, theme::Theme};

struct BenchmarkStats {
    min: Duration,
    max: Duration,
    mean: Duration,
    median: Duration,
    p95: Duration,
    samples: Vec<Duration>,
}

impl BenchmarkStats {
    fn new(mut samples: Vec<Duration>) -> Self {
        samples.sort();
        let len = samples.len();
        let mean = samples.iter().sum::<Duration>() / len as u32;
        let median = samples[len / 2];
        let p95 = samples[(len as f64 * 0.95) as usize];
        let min = samples[0];
        let max = samples[len - 1];

        Self {
            min,
            max,
            mean,
            median,
            p95,
            samples,
        }
    }

    fn report(&self, name: &str) {
        eprintln!("\nBenchmark Results for {}", name);
        eprintln!("---------------------------");
        eprintln!("Min:    {:?}", self.min);
        eprintln!("Max:    {:?}", self.max);
        eprintln!("Mean:   {:?}", self.mean);
        eprintln!("Median: {:?}", self.median);
        eprintln!("P95:    {:?}", self.p95);

        let mean_nanos = self.mean.as_nanos() as f64;
        let variance: f64 = self
            .samples
            .iter()
            .map(|&d| {
                let diff = d.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>()
            / self.samples.len() as f64;
        let std_dev = Duration::from_nanos(variance.sqrt() as u64);
        eprintln!("StdDev: {:?}", std_dev);
    }
}

fn main() {
    let _ = Theme::new(&Config::default());
    let themes = termitype::theme::available_themes();

    benchmark_theme_switching(themes);

    benchmark_theme_rapid_switching(themes);
}

fn benchmark_theme_switching(themes: &[String]) {
    const WARMUP_ITERATIONS: usize = 100;
    const MEASURE_ITERATIONS: usize = 1000;

    eprintln!("\nWarming up...");
    for theme_name in themes.iter().cycle().take(WARMUP_ITERATIONS) {
        let _ = Theme::from_name(theme_name);
    }

    eprintln!("Collecting measurements...");
    let mut measurements = Vec::with_capacity(MEASURE_ITERATIONS);

    for theme_name in themes.iter().cycle().take(MEASURE_ITERATIONS) {
        std::thread::sleep(Duration::from_micros(10));

        let start = Instant::now();
        let _ = Theme::from_name(theme_name);
        measurements.push(start.elapsed());
    }

    let stats = BenchmarkStats::new(measurements);
    stats.report("Theme Switching");

    assert!(
        stats.p95 < Duration::from_micros(500),
        "95th percentile theme switching time ({:?}) exceeded 500µs threshold",
        stats.p95
    );
}

fn benchmark_theme_rapid_switching(themes: &[String]) {
    const BURST_SIZE: usize = 100;
    const NUM_BURSTS: usize = 50;

    let mut burst_times = Vec::with_capacity(NUM_BURSTS);

    eprintln!("\nMeasuring rapid theme switching in bursts...");
    for i in 0..NUM_BURSTS {
        if i > 0 {
            std::thread::sleep(Duration::from_millis(100));
        }

        let start = Instant::now();
        for theme_name in themes.iter().cycle().take(BURST_SIZE) {
            let _ = Theme::from_name(theme_name);
        }
        burst_times.push(start.elapsed());
    }

    let stats = BenchmarkStats::new(burst_times);
    stats.report(&format!("Rapid Theme Switching (bursts of {BURST_SIZE})"));

    assert!(
        (stats.mean / BURST_SIZE as u32) < Duration::from_micros(200),
        "Average theme switch time in burst ({:?}) exceeded 200µs threshold",
        stats.mean / BURST_SIZE as u32
    );
}
