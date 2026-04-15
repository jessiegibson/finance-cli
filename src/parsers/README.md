# src/parsers/

Import pipeline for bank transaction files. Detects file format and institution, then parses rows into `Transaction` domain models.

## Files

### mod.rs
Module root. Declares `FileFormat` and `ParseResult` types and lists the eight supported institutions: Chase, Bank of America, Wealthfront, Ally, American Express, Discover, Citi, Capital One.

### detect.rs
Format and institution detection. `FileFormat` distinguishes Csv, Qfx, Ofx, and Unknown. `Institution` enumerates supported banks. Detection inspects file extensions and CSV headers to pick the right column mapping.

### csv.rs
CSV parser for bank exports. `parse_csv_file` reads the file, calls `detect_institution`, and applies per-bank column mappings to build `Transaction` rows using `TransactionBuilder`. Handles date formats, amount signs, and description normalization unique to each institution.

### qfx.rs
QFX/OFX parser. Uses `quick-xml` to walk the SGML-flavored tree and extract `STMTTRN` entries, mapping `DTPOSTED`, `TRNAMT`, `NAME`, and `FITID` fields into `Transaction` rows.
