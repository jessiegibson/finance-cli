# benches/fixtures/

Deterministic synthetic fixtures used by `parser_bench.rs`. Every file contains 100 transactions drawn from a seeded random merchant list (`random.seed(42)` in the generator), so benchmark runs stay comparable across builds.

## CSV fixtures

Each file matches the column layout the parser expects for its institution (see `src/parsers/detect.rs::CsvMapping`).

- `chase.csv` — 6 columns: `Details, Posting Date (MM/DD/YYYY), Description, Amount, Type, Balance`. Used to exercise the Chase mapping with its category column.
- `bank_of_america.csv` — 4 columns: `Date (MM/DD/YYYY), Description, Amount, Running Bal.`. Used for the BofA mapping.
- `wealthfront.csv` — 3 columns: `Date (YYYY-MM-DD), Amount, Description`. Used for the Wealthfront mapping.
- `american_express.csv` — 3 columns: `Date (MM/DD/YYYY), Description, Amount`. Expenses are positive, income negative, exercising the `negate_amounts` flag.
- `ally.csv`, `discover.csv`, `citi.csv`, `capital_one.csv` — 3 columns: `Date (YYYY-MM-DD), Amount, Description`. These hit the generic fallback mapping.

## QFX fixture

- `generic.qfx` — Full OFX SGML document with the standard header, one `BANKMSGSRSV1/STMTTRNRS/STMTRS` envelope, and 100 `STMTTRN` blocks. Used for the `parse_qfx_content` benchmark.

## Regenerating fixtures

The fixtures were produced by a Python script committed to the repository history (see the PR that added this directory). Re-running the generator with the same seed reproduces identical files. Do not edit fixtures by hand. Instead, update the generator and regenerate the entire set so all banks stay in sync.

## Row count

100 rows per fixture balances two goals. Large enough to amortize per-call overhead in Criterion samples, small enough that each bench iteration stays under a millisecond so Criterion converges without running for minutes. If you need to measure scaling behavior, add a separate parametric benchmark rather than inflating these files.
