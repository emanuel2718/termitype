use clap::Parser;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use termitype::{config::Mode, tracker::Tracker};

#[derive(Parser)]
#[command(name = "termitype-simulation")]
struct Cli {
    #[arg(short, long, default_value = "2.0")]
    wps: f64,

    #[arg(short, long, default_value = "3")]
    iterations: usize,

    #[arg(short = 't', long, default_value = "30")]
    duration: usize,

    #[arg(short, long, default_value = "0.05")]
    error_rate: f64,

    #[arg(short = 'T', long)]
    text: Option<String>,

    #[arg(short, long)]
    verbose: bool,

    #[arg(long)]
    live: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct SimulationResult {
    // text_length: usize,
    text: String,
    wpm: f64,
    net_wpm: f64,
    accuracy: f64,
    consistency: f64,
    iteration: usize,
    target_wps: f64,
    actual_wps: f64,
    total_errors: usize,
    elapsed_time_ms: u128,
}

#[derive(Serialize, Deserialize)]
struct SimulationSummary {
    total_iterations: usize,
    average_wps: f64,
    average_wpm: f64,
    average_accuracy: f64,
    average_net_wpm: f64,
    average_consistency: f64,
    min_wps: f64,
    max_wps: f64,
    std_dev_wps: f64,
    total_time_ms: u128,
    results: Vec<SimulationResult>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.wps <= 0.0 {
        eprintln!("Error: WPS must be greater than 0");
        std::process::exit(1);
    }

    if cli.error_rate < 0.0 || cli.error_rate > 1.0 {
        eprintln!("Error: Error rate must be between 0.0 and 1.0");
        std::process::exit(1);
    }

    let default_texts = vec![
        "The quick brown fox jumps over the lazy dog",
        "Ex voluptate commodo irure est nostrud laborum quis.",
        "Quis incididunt ad deserunt velit ullamco tempor laborum commodo enim velit id.",
        "Cupidatat nostrud id laborum in dolor id laborum.",
        "Commodo labore in ad voluptate amet nulla Lorem anim ipsum nulla nulla minim exercitation.",
    ];

    let texts: Vec<String> = if let Some(ref custom_text) = cli.text {
        vec![custom_text.clone()]
    } else {
        default_texts.into_iter().map(String::from).collect()
    };

    let mut all_results = Vec::new();
    let simluation_start = Instant::now();

    println!("$ termitype-simulation");
    println!();
    println!("Iterations: {}", cli.iterations);
    println!("Target WPS: {:.2}", cli.wps);
    println!("Duration: {}s", cli.duration);
    println!("Error Rate: {:.1}%", cli.error_rate * 100.0);
    println!("Texts: {}", texts.len());
    println!();

    for iteration in 0..cli.iterations {
        let text = &texts[iteration % texts.len()];
        let result = simulate(
            text,
            cli.wps,
            cli.duration,
            cli.error_rate,
            iteration,
            cli.live,
        )?;

        all_results.push(result);
    }

    let summary = calculate_summary(&all_results, simluation_start.elapsed().as_millis());

    display_simulation_summary(&summary, &cli);

    Ok(())
}

fn simulate(
    text: &str,
    wps: f64,
    duration: usize,
    error_rate: f64,
    iteration: usize,
    live: bool,
) -> Result<SimulationResult, Box<dyn std::error::Error>> {
    let mode = Mode::with_time(duration);
    let mut tracker = Tracker::new(text.to_string(), mode);

    let chars_per_second = wps * 5.0; // on average theres 5 characters per word
    let char_delay_ms = ((1.0 / chars_per_second) * 1000.0) as u64;

    tracker.start_typing();
    let start_time = Instant::now();

    if live {
        print!("\x1b[2J\x1b[H"); // clear screen && move cursor
        println!("$ termitype-simulation --live");
        println!();
        println!("Iteration: {} | Target WPS: {:.1}", iteration + 1, wps);
        println!("Text Size: {}", text.len());
        println!();
    }

    let chars: Vec<char> = text.chars().collect();
    for &expected_char in chars.iter() {
        let char_to_type = if rand::random::<f64>() < error_rate {
            // random wrong character
            (b'a' + (rand::random::<u8>() % 26)) as char
        } else {
            expected_char
        };

        tracker.type_char(char_to_type)?;

        // live visualization
        if live {
            display_live_progress(&mut tracker, text, wps, iteration);
        }

        std::thread::sleep(std::time::Duration::from_millis(char_delay_ms));
    }

    let elapsed = start_time.elapsed();
    let summary = tracker.summary();

    Ok(SimulationResult {
        iteration,
        target_wps: wps,
        actual_wps: summary.wps,
        wpm: summary.wpm,
        accuracy: summary.accuracy,
        total_errors: summary.total_errors,
        elapsed_time_ms: elapsed.as_millis(),
        text: text.to_string(),
        net_wpm: summary.net_wpm(),
        consistency: summary.consistency,
    })
}

fn calculate_summary(results: &[SimulationResult], total_time_ms: u128) -> SimulationSummary {
    let total_iterations = results.len();

    if results.is_empty() {
        return SimulationSummary {
            total_iterations: 0,
            average_wps: 0.0,
            average_wpm: 0.0,
            average_accuracy: 0.0,
            average_net_wpm: 0.0,
            average_consistency: 0.0,
            min_wps: 0.0,
            max_wps: 0.0,
            std_dev_wps: 0.0,
            total_time_ms,
            results: vec![],
        };
    }

    let sum_wps: f64 = results.iter().map(|r| r.actual_wps).sum();
    let sum_wpm: f64 = results.iter().map(|r| r.wpm).sum();
    let sum_accuracy: f64 = results.iter().map(|r| r.accuracy).sum();
    let sum_net_wpm: f64 = results.iter().map(|r| r.net_wpm).sum();
    let sum_consistency: f64 = results.iter().map(|r| r.consistency).sum();

    let average_wps = sum_wps / total_iterations as f64;
    let average_wpm = sum_wpm / total_iterations as f64;
    let average_accuracy = sum_accuracy / total_iterations as f64;
    let average_net_wpm = sum_net_wpm / total_iterations as f64;
    let average_consistency = sum_consistency / total_iterations as f64;

    let min_wps = results
        .iter()
        .map(|r| r.actual_wps)
        .fold(f64::INFINITY, f64::min);
    let max_wps = results
        .iter()
        .map(|r| r.actual_wps)
        .fold(f64::NEG_INFINITY, f64::max);

    let variance = results
        .iter()
        .map(|r| (r.actual_wps - average_wps).powi(2))
        .sum::<f64>()
        / total_iterations as f64;
    let std_dev_wps = variance.sqrt();

    SimulationSummary {
        total_iterations,
        average_wps,
        average_wpm,
        average_accuracy,
        average_net_wpm,
        average_consistency,
        min_wps,
        max_wps,
        std_dev_wps,
        total_time_ms,
        results: results.to_vec(),
    }
}

fn display_simulation_summary(summary: &SimulationSummary, cli: &Cli) {
    println!("$ termitype-simulation --summary");
    println!();
    println!("Total Iterations: {}", summary.total_iterations);
    println!(
        "Average WPS: {:.3} (target: {:.2})",
        summary.average_wps, cli.wps
    );
    println!("Average WPM: {:.1}", summary.average_wpm);
    println!("Average Accuracy: {:.1}%", summary.average_accuracy * 100.0);
    println!("Average Net WPM: {:.1}", summary.average_net_wpm);
    println!("Average Consistency: {:.3}", summary.average_consistency);
    println!("WPS Range: {:.3} - {:.3}", summary.min_wps, summary.max_wps);
    println!("WPS Standard Deviation: {:.3}", summary.std_dev_wps);
    println!(
        "Total Simulation Time: {:.2}s",
        summary.total_time_ms as f64 / 1000.0
    );
    println!();

    if cli.verbose {
        println!("$ termitype-simulation --results");
        println!();
        for result in &summary.results {
            println!(
                "Iteration #{:1}: WPM={:.1}, WPS={:.3}, Acc={:.1}%, Net WPM={:.1}, Err={}",
                result.iteration + 1,
                result.wpm,
                result.actual_wps,
                result.accuracy * 100.0,
                result.net_wpm,
                result.total_errors
            );
        }
        println!();
    }
}

fn display_live_progress(state: &mut Tracker, text: &str, target_wps: f64, iteration: usize) {
    // move cursor to the top of the screen
    print!("\x1b[H");

    println!("$ termitype-simulation --live");
    println!();
    println!(
        "Iteration #{} | Target WPS: {:.1}",
        iteration + 1,
        target_wps
    );
    println!("Text Size: {}", text.len());
    println!();

    print!("Progress: ");
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if i < state.current_pos {
            if let Some(token) = state.tokens.get(i) {
                if token.is_wrong {
                    print!("\x1b[31m{}\x1b[0m", c); // red
                } else {
                    print!("\x1b[32m{}\x1b[0m", c); // green
                }
            }
        } else if i == state.current_pos && state.current_pos < chars.len() {
            print!("\x1b[33m{}\x1b[0m\x1b[33m▊\x1b[0m", c); // yellow cursor
        } else {
            print!("{}", c); // default color
        }
    }
    println!();
    println!();

    let summary = state.summary();
    println!("WPS: {:.3}", summary.wps);
    println!("WPM: {:.1}", summary.wpm);
    println!("Net WPM: {:.1}", summary.net_wpm());
    println!("Accuracy: {:.1}%", summary.accuracy * 100.0);
    println!("Progress: {:.1}%", summary.progress * 100.0);
    println!("Consistency: {:.3}", summary.consistency);
    println!("Errors: {}", summary.total_errors);
    println!("Time: {:.2}s", summary.elapsed_time.as_secs_f64());
    println!();

    // flush it out
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}
