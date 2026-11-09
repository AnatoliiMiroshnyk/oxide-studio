# Test strategy

## 1. Quality objectives

Tests must establish that database operations are correct, predictable under malformed input, safe around user files, and bounded in their impact on memory and responsiveness. UI tests should verify workflows, not only that individual widgets render.

## 2. Test layers

| Layer | Scope | Examples | Status in v0.1.0 |
|---|---|---|---|
| Unit | Pure conversion and SQL construction helpers | Identifier quoting, value formatting, CSV serialization | Partial |
| Database unit | One in-memory SQLite connection | Schema enumeration, table metadata, query result mapping, read-only writes | Partial |
| Integration | Temporary on-disk DBs and sidecar files | Open/create/read-only, WAL, locked DB, corrupt file, export permissions | Planned |
| UI component | Isolated egui frame/state tests | Empty states, tab transitions, keyboard shortcuts, disabled controls | Planned |
| End-to-end | Built application on target OS | Open DB, browse, run SQL, export CSV, close/reopen | Planned |
| Performance | Large and adversarial fixtures | Wide tables, large text, many rows, slow query, large BLOB | Planned |
| Security | Fault injection, fuzzing, dependency review | Corrupt DB, unusual identifiers, malformed UTF-8, import parser fuzzing | Planned |

## 3. Required fixture matrix

Keep deterministic fixtures for:

- Empty database and database with no user tables.
- Tables with composite primary keys, generated columns, `WITHOUT ROWID`, and foreign keys.
- Views, indexes, triggers, and names containing spaces, quotes, Unicode, and SQL keywords.
- NULL, all integer/real/text/blob value types, invalid UTF-8 text where SQLite permits it, and very large values.
- Virtual tables / FTS when enabled in the bundled build.
- WAL-mode databases, locked databases, read-only file permissions, and corrupt/truncated database files.
- CSV cells containing delimiters, quotes, CRLF, embedded newlines, and empty/null values.

## 4. Database correctness tests

- Assert that identifiers are escaped rather than interpolated raw.
- Assert pagination values are bound parameters and page boundaries do not duplicate rows.
- Assert row and text limits are visible in result metadata and UI copy.
- Assert query errors do not replace the previous successful result unexpectedly.
- Assert read-only connections reject normal write statements.
- Assert a failed write transaction rolls back all prior statements in that transaction.
- Assert exports are byte-for-byte valid for RFC 4180-sensitive content and clearly identify partial materialization.

## 5. Performance tests

Use generated fixtures at 10k, 100k, and 1m rows, plus wide tables and large text/BLOB columns. Track:

- cold startup time and time to first interactive frame;
- schema discovery time;
- first-page and deep-page browse time;
- query first-row and full-preview latency;
- peak resident memory and number of display cells;
- cancellation latency and UI frame latency during active database work.

Do not accept a benchmark improvement that weakens correctness, type fidelity, or data-safety guarantees. Store benchmark hardware, OS, SQLite build, dataset generator seed, and commit SHA with every result.

## 6. CI gates

Pull requests should run formatting, Clippy with warnings denied, unit/integration tests, and a platform compile matrix. The 1.0 pipeline should additionally run dependency vulnerability and license checks, an SBOM generator, selected fuzz targets, and reproducible release-package checks.

## 7. Current baseline

The prototype includes unit tests for opening the demo database, basic query mapping, schema/table metadata, identifier quoting, read-only mutation rejection, and CSV quoting. In the generation environment, the Rust toolchain was unavailable, so these tests have not been executed here. The demo SQL fixture itself was executed using Python's SQLite driver and passed schema/data/integrity smoke checks; this does not validate Rust compilation.
