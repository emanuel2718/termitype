use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::num::NonZeroUsize;
use termitype::{
    builders::lexicon_builder::LexiconBuilder,
    config::{self, Config},
    theme::Theme,
};

fn bench_word_pool(c: &mut Criterion) {
    let mut builder = LexiconBuilder::new();

    let mut group = c.benchmark_group("word_pool");

    group.bench_function("default_config", |b| {
        b.iter(|| {
            let config = Config::default();
            let _ = black_box(builder.generate_test(&config));
        })
    });

    group.bench_function("words_mode_50", |b| {
        b.iter(|| {
            let config = Config::default();
            let _ = black_box(builder.generate_test(&config));
        })
    });

    group.bench_function("time_mode_60s", |b| {
        b.iter(|| {
            let config = Config::default();
            let _ = black_box(builder.generate_test(&config));
        })
    });

    group.bench_function("words 10k", |b| {
        b.iter(|| {
            let mut config = Config::default();
            let _ = config.change_mode(config::Mode::Words(NonZeroUsize::new(10_000).unwrap()));
            let _ = black_box(builder.generate_test(&config));
        });
    });

    group.finish();
}

fn bench_themes(c: &mut Criterion) {
    let themes = termitype::theme::available_themes();

    let mut group = c.benchmark_group("themes");

    group.bench_function("theme_switching", |b| {
        b.iter(|| {
            for theme_name in themes.iter().cycle().take(1) {
                let _ = black_box(theme_name.parse::<Theme>());
            }
        })
    });

    group.bench_function("rapid_theme_switching_burst_100", |b| {
        b.iter(|| {
            for theme_name in themes.iter().cycle().take(100) {
                let _ = black_box(theme_name.parse::<Theme>());
            }
        })
    });

    group.finish();
}

fn bench(c: &mut Criterion) {
    bench_word_pool(c);
    bench_themes(c);
}

criterion_group!(benches, bench);
criterion_main!(benches);
