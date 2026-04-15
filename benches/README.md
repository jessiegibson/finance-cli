# benches/

Criterion benchmarks run with `cargo bench`. HTML reports land in `target/criterion/report/index.html`.

## Files

### parser_bench.rs
Measures CSV and QFX parsing throughput against realistic fixtures. Defines four benchmark groups:

- `parse_csv_by_bank` — Per-bank CSV parse time measured in bytes per second. One entry per institution (Chase, Bank of America, Wealthfront, Ally, American Express, Discover, Citi, Capital One).
- `parse_csv_rows_per_sec` — Same workload, throughput reported in rows per second for a transaction-oriented view.
- `parse_qfx` — QFX/OFX SGML parser throughput over a 100-row fixture, reported in both bytes and rows per second.
- `detect_institution` — Institution detection heuristics alone, isolated from parse work, for catching regressions in the keyword matcher.

All fixtures are embedded at compile time via `include_str!` so the timing loop never touches disk.

### categorization_bench.rs
Benchmark harness for the categorization engine. Currently a placeholder. Populate with realistic transaction and rule sets to measure rule-matching throughput and end-to-end categorization latency.

## Subdirectories

- `fixtures/` — Deterministic synthetic bank exports used by `parser_bench.rs`. See `fixtures/README.md` for format details and row counts.

## Running

    cargo bench --bench parser_bench
    cargo bench --bench parser_bench -- parse_csv_by_bank/chase
    cargo bench --bench parser_bench -- detect_institution

The second command runs only the Chase entry of the `parse_csv_by_bank` group. The third runs the detection-only group. Criterion filter strings are substring matches against the benchmark path.
