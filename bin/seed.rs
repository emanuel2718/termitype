use chrono::Local;
use clap::Parser;
use rand::Rng;
use termitype::{
    constants::db_file,
    db::{Db, LeaderboardResult},
    error::AppResult,
};

#[derive(Parser)]
#[command(name = "termitype-seed")]
struct Cli {
    /// Number of dummy entries to seed into the database
    #[arg(long, value_name = "N")]
    seed: Option<usize>,

    /// Clear all entries from the database
    #[arg(long)]
    clear: bool,
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();

    let mut db = Db::new(db_file())?;

    if cli.clear {
        let deleted = db.reset()?;
        println!("Cleared database: deleted {} entries", deleted);
    }

    if let Some(n) = cli.seed {
        for _ in 0..n {
            let result = generate_dummy_result();
            db.insert_dummy_result(result)?;
        }
        println!("Seeded {} dummy entries into the database", n);
    }

    Ok(())
}

fn generate_dummy_result() -> LeaderboardResult {
    let mut rng = rand::rng();

    let mode_kinds = ["Time", "Words"];
    let mode_kind = mode_kinds[rng.random_range(0..2)];

    let mode_value = if mode_kind == "Time" {
        rng.random_range(15..=300)
    } else {
        rng.random_range(10..=1_000)
    };

    let languages = [
        "english",
        "spanish",
        "latin",
        "code_python",
        "code_rust",
        "code_typescript",
    ];
    let language = languages[rng.random_range(0..languages.len())].to_string();

    let wpm = rng.random_range(20..=350) as u16;
    let raw_wpm = wpm + rng.random_range(10..=25);
    let accuracy = rng.random_range(30..=100) as u16;
    let consistency = rng.random_range(30..=100) as u16;
    let error_count = rng.random_range(0..=100) as u32;

    let numbers = rng.random_bool(0.3);
    let symbols = rng.random_bool(0.2);
    let punctuation = rng.random_bool(0.2);

    let created_at = Local::now() - chrono::Duration::days(rng.random_range(0..30));

    LeaderboardResult {
        id: None,
        mode_kind: mode_kind.to_string(),
        mode_value: mode_value as i32,
        language,
        wpm,
        raw_wpm,
        accuracy,
        consistency,
        error_count,
        numbers,
        symbols,
        punctuation,
        created_at,
    }
}
