//! Parser benchmarks.
//!
//! Measures CSV and QFX parsing throughput against realistic per-bank fixtures
//! stored in `benches/fixtures/`. Each fixture holds 100 deterministic rows so
//! results stay comparable across runs.
//!
//! Run with:
//!
//!     cargo bench --bench parser_bench
//!
//! HTML report lands in `target/criterion/report/index.html`.

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use finance_cli::models::{Account, AccountType};
use finance_cli::parsers::{csv::parse_csv_content, detect_institution, qfx::parse_qfx_content};

// Embed fixtures at compile time so benchmarks never touch the filesystem
// during measurement. This keeps the timing loop focused on parse work.
const CHASE: &str = include_str!("fixtures/chase.csv");
const BOFA: &str = include_str!("fixtures/bank_of_america.csv");
const WEALTHFRONT: &str = include_str!("fixtures/wealthfront.csv");
const ALLY: &str = include_str!("fixtures/ally.csv");
const AMEX: &str = include_str!("fixtures/american_express.csv");
const DISCOVER: &str = include_str!("fixtures/discover.csv");
const CITI: &str = include_str!("fixtures/citi.csv");
const CAPITAL_ONE: &str = include_str!("fixtures/capital_one.csv");
const QFX: &str = include_str!("fixtures/generic.qfx");

/// Per-bank CSV fixture metadata.
struct CsvFixture {
    name: &'static str,
    hint: &'static str,
    content: &'static str,
}

const CSV_FIXTURES: &[CsvFixture] = &[
    CsvFixture { name: "chase", hint: "chase", content: CHASE },
    CsvFixture { name: "bank_of_america", hint: "bofa", content: BOFA },
    CsvFixture { name: "wealthfront", hint: "wealthfront", content: WEALTHFRONT },
    CsvFixture { name: "ally", hint: "ally", content: ALLY },
    CsvFixture { name: "american_express", hint: "amex", content: AMEX },
    CsvFixture { name: "discover", hint: "discover", content: DISCOVER },
    CsvFixture { name: "citi", hint: "citi", content: CITI },
    CsvFixture { name: "capital_one", hint: "capital_one", content: CAPITAL_ONE },
];

fn bench_account() -> Account {
    Account::new("Bench", "Bench Bank", AccountType::Checking)
}

/// Per-bank CSV parse throughput.
fn bench_csv_by_bank(c: &mut Criterion) {
    let account = bench_account();
    let mut group = c.benchmark_group("parse_csv_by_bank");

    for fixture in CSV_FIXTURES {
        group.throughput(Throughput::Bytes(fixture.content.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(fixture.name),
            fixture,
            |b, fx| {
                b.iter(|| {
                    let result = parse_csv_content(
                        black_box(fx.content),
                        black_box(&account),
                        black_box(Some(fx.hint)),
                    )
                    .expect("fixture should parse cleanly");
                    // Sanity check once per iteration: every fixture is sized at
                    // 100 rows. If this ever drops below 99 the fixture regressed.
                    debug_assert!(result.transactions.len() >= 99);
                    result
                })
            },
        );
    }

    group.finish();
}

/// CSV parse throughput measured in rows per second. Same work as
/// `bench_csv_by_bank`, different throughput unit for a rows-oriented view.
fn bench_csv_rows_throughput(c: &mut Criterion) {
    let account = bench_account();
    let mut group = c.benchmark_group("parse_csv_rows_per_sec");

    for fixture in CSV_FIXTURES {
        group.throughput(Throughput::Elements(100));
        group.bench_with_input(
            BenchmarkId::from_parameter(fixture.name),
            fixture,
            |b, fx| {
                b.iter(|| {
                    parse_csv_content(
                        black_box(fx.content),
                        black_box(&account),
                        black_box(Some(fx.hint)),
                    )
                    .expect("fixture should parse cleanly")
                })
            },
        );
    }

    group.finish();
}

/// QFX parse throughput over the 100-row OFX SGML fixture.
fn bench_qfx(c: &mut Criterion) {
    let account = bench_account();
    let mut group = c.benchmark_group("parse_qfx");

    group.throughput(Throughput::Bytes(QFX.len() as u64));
    group.bench_function("generic_100_rows", |b| {
        b.iter(|| {
            let result = parse_qfx_content(black_box(QFX), black_box(&account))
                .expect("qfx fixture should parse cleanly");
            debug_assert!(result.transactions.len() >= 99);
            result
        })
    });

    group.throughput(Throughput::Elements(100));
    group.bench_function("generic_100_rows_rps", |b| {
        b.iter(|| {
            parse_qfx_content(black_box(QFX), black_box(&account))
                .expect("qfx fixture should parse cleanly")
        })
    });

    group.finish();
}

/// Institution detection alone, separate from parse work. Useful for
/// catching regressions in the keyword heuristics in `detect.rs`.
fn bench_detect_institution(c: &mut Criterion) {
    let mut group = c.benchmark_group("detect_institution");

    for fixture in CSV_FIXTURES {
        group.throughput(Throughput::Bytes(fixture.content.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(fixture.name),
            fixture,
            |b, fx| {
                b.iter(|| detect_institution(black_box(fx.content)));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_csv_by_bank,
    bench_csv_rows_throughput,
    bench_qfx,
    bench_detect_institution,
);
criterion_main!(benches);
