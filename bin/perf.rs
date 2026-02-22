use clap::Parser;
use std::time::Instant;
use termitype::{
    config::Mode, theme::Theme, tracker::Tracker, tui::components::typing_cache::TypingRenderCache,
};

#[derive(Parser, Debug)]
#[command(
    name = "termitype-perf",
    about = "Synthetic performance micro-benchmarks."
)]
struct Cli {
    #[arg(long, default_value_t = 5000)]
    iterations: usize,

    #[arg(long, default_value_t = 80)]
    width: u16,

    #[arg(long, default_value_t = 50)]
    words: usize,

    #[arg(long, default_value_t = false)]
    tracker_only: bool,

    #[arg(long, default_value_t = false)]
    render_only: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let text = create_text(cli.words);

    if !cli.render_only {
        benchmark_tracker(&text, cli.iterations);
    }
    if !cli.tracker_only {
        benchmark_render(&text, cli.iterations, cli.width);
    }

    Ok(())
}

fn benchmark_tracker(text: &str, iterations: usize) {
    let chars: Vec<char> = text.chars().collect();
    let started_at = Instant::now();

    for _ in 0..iterations {
        let mut tracker = Tracker::new(
            text.to_string(),
            Mode::with_words(text.split_whitespace().count()),
        );
        for c in chars.iter().copied() {
            let _ = tracker.type_char(c);
        }
    }

    let elapsed = started_at.elapsed();
    let total_chars = chars.len() * iterations;
    println!("tracker:");
    println!("  iterations: {}", iterations);
    println!("  total chars: {}", total_chars);
    println!("  elapsed: {:.3}s", elapsed.as_secs_f64());
    println!(
        "  chars/sec: {:.0}",
        total_chars as f64 / elapsed.as_secs_f64()
    );
}

fn benchmark_render(text: &str, iterations: usize, width: u16) {
    let theme = Theme::default();
    let mut tracker = Tracker::new(
        text.to_string(),
        Mode::with_words(text.split_whitespace().count()),
    );
    for c in text.chars() {
        let _ = tracker.type_char(c);
    }

    let mut cache = TypingRenderCache::default();
    let started_at = Instant::now();

    for revision in 0..iterations {
        cache.ensure(&tracker, &theme, width, 3, revision as u64);
    }

    let elapsed = started_at.elapsed();
    println!("render:");
    println!("  iterations: {}", iterations);
    println!("  elapsed: {:.3}s", elapsed.as_secs_f64());
    println!(
        "  rebuilds/sec: {:.0}",
        iterations as f64 / elapsed.as_secs_f64()
    );
}

fn create_text(words: usize) -> String {
    const BASE: &[&str] = &[
        "the",
        "quick",
        "brown",
        "fox",
        "jumps",
        "over",
        "the",
        "lazy",
        "dog",
        "typing",
        "speed",
        "accuracy",
        "rhythm",
        "focus",
        "flow",
        "consistency",
    ];

    (0..words)
        .map(|i| BASE[i % BASE.len()])
        .collect::<Vec<_>>()
        .join(" ")
}
