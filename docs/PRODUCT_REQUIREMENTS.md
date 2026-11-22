# Product requirements — Oxide Studio

**Document status:** v0.2.0 implementation baseline; requirements for 1.0 are targets, not implemented claims.

## 1. Product definition

Oxide Studio is a local-first desktop workbench for SQLite. It should make routine database exploration, SQL development, data changes, diagnostics, and export comfortable without requiring a server, account, browser runtime, or separate database installation.

The product is intended as a credible alternative to SQLiteStudio, not as a reimplementation of its legacy interface. The experience should be task-oriented: a searchable object explorer, focused data and structure inspectors, a multi-tab SQL workbench, and explicit operation feedback.

## 2. Goals and non-goals

### Goals

- Support the normal SQLite development lifecycle: open, inspect, query, edit, import/export, diagnose, back up, and compare.
- Keep database files local by default and make all external communication opt-in and visible.
- Keep startup, idle memory, and common browsing workflows efficient on ordinary developer hardware.
- Prefer safe defaults: read-only inspection, bounded result sets, explicit destructive-operation handling, and recoverable write workflows.
- Provide first-class Windows, macOS, and Linux desktop builds.
- Keep the database core independently testable from the UI.

### Non-goals for the first major release

- Acting as a database server or multi-user collaboration service.
- Replacing general-purpose spreadsheet software.
- Supporting arbitrary third-party native plugins before a permission model exists.
- Automatically rewriting user SQL or silently applying schema migrations.

## 3. Target users and primary jobs

| Persona | Main job | Success signal |
|---|---|---|
| Application developer | Inspect a local application database and run SQL while debugging | Open a database and answer a data question in seconds |
| Data analyst | Explore tables/views and export query results | Reproducible result preview and correct CSV export |
| Database maintainer | Inspect schema, indexes, constraints, and integrity | Understand schema and diagnose issues without modifying data accidentally |
| Learner / educator | Explore SQLite concepts and test SQL safely | Demo workspace works with zero setup and read-only mode is obvious |

## 4. Target core workflows (1.0)

1. **Open and inspect:** choose a database file, inspect object counts, find a table, browse a bounded page, switch to structure, and copy or export results.
2. **Query:** open a SQL tab, write a statement, bind typed parameters, execute, inspect result metadata, cancel a long query, and export all result rows through a streaming path.
3. **Edit safely:** select rows, preview insert/update/delete changes, apply them in an explicit transaction, and roll back on validation or constraint errors.
4. **Change schema:** create or alter supported SQLite objects, preview generated SQL, apply the operation, and show the resulting schema diff.
5. **Move data:** import CSV/JSON with preview and type/null handling, export query or selected table data, and create SQL dumps.
6. **Maintain:** inspect pragmas, foreign-key violations, query plans, run integrity checks, checkpoint WAL, and create/restore backups.

## 5. Functional requirements

Priority: **P0** = needed for a usable 1.0; **P1** = important post-1.0; **P2** = optional/advanced.

| ID | Requirement | Priority | v0.2.0 status |
|---|---|---:|---|
| DB-01 | Open existing SQLite database files | P0 | Implemented |
| DB-02 | Create a new SQLite database file | P0 | Implemented |
| DB-03 | Open a database in read-only mode | P0 | Implemented; not a security sandbox for arbitrary SQL |
| DB-04 | Keep several database connections/workspaces open | P0 | Not implemented |
| DB-05 | Show effective SQLite version, journal mode, and connection settings | P1 | SQLite version only |
| SC-01 | Browse tables, views, indexes, and triggers | P0 | Implemented |
| SC-02 | Inspect column declarations, nullability, defaults, primary keys, and DDL | P0 | Basic implementation |
| SC-03 | Search/filter schema objects | P0 | Name filter implemented |
| SC-04 | Create and perform supported rename/add/drop schema operations | P0 | Guided subset implemented; no generic table rebuild |
| SC-05 | Visualize foreign-key relationships and ER diagrams | P1 | Not implemented |
| DATA-01 | Browse rows with pagination | P0 | Implemented, 200 rows/page |
| DATA-02 | Sort, filter, search, and select columns | P0 | Not implemented |
| DATA-03 | Insert, edit, and delete rows | P0 | Guided forms implemented; no duplicate-row action |
| DATA-04 | Preserve typed NULL, integer, real, text, and BLOB values | P0 | Owned typed values for edit pages; numeric affinity parsing and hex BLOB editor |
| DATA-05 | Identify records using primary keys, composite keys, or safe rowid fallback | P0 | Implemented; rows with no safe identity are browse-only |
| SQL-01 | Execute SELECT, DML, DDL, and PRAGMA statements | P0 | One prepared statement/run |
| SQL-02 | Multi-tab SQL editor with saved queries | P0 | Session history only |
| SQL-03 | Syntax highlighting, completion, formatting, and parameter binding | P0 | Not implemented |
| SQL-04 | Show execution time, affected rows, errors, and bounded output | P0 | Basic implementation |
| SQL-05 | Cancel long-running queries and report progress | P0 | Not implemented; synchronous UI execution |
| TX-01 | Start DEFERRED, IMMEDIATE, and EXCLUSIVE transactions | P0 | Implemented |
| TX-02 | Commit and confirmation-gated rollback | P0 | Implemented |
| TX-03 | Execute guided mutations inside recoverable savepoints | P0 | Implemented; savepoint rollback failure is logged |
| IO-01 | Export query results to CSV | P0 | Current materialized rows only |
| IO-02 | Import CSV with preview and type/null mapping | P0 | Not implemented |
| IO-03 | JSON and SQL dump import/export | P1 | Not implemented |
| IO-04 | Stream large exports without materializing all rows | P0 | Not implemented |
| OPS-01 | Run SQLite integrity and foreign-key checks | P0 | Integrity check implemented |
| OPS-02 | Explain query plans and inspect pragma values | P0 | User can run SQL manually |
| OPS-03 | Backup and restore using SQLite's backup facilities | P0 | Not implemented |
| OPS-04 | VACUUM, ANALYZE, REINDEX, and WAL checkpoint workflows | P1 | Not implemented |
| UX-01 | Keyboard navigation, command/search palette, and configurable shortcuts | P1 | Ctrl+Enter only |
| UX-02 | Dark/light/high-contrast themes | P1 | Custom dark theme |
| UX-03 | Persistent recent files, layout, and workspace state | P0 | Not implemented |
| EXT-01 | Plugin API with explicit permissions/capability isolation | P2 | Not implemented |

## 6. 0.2.0 scope clarifications

The 0.2.0 implementation deliberately does not claim a generic schema-alter engine. SQLite schema changes that require reconstructing a table are deferred until the product can preview and preserve constraints, indexes, triggers, generated columns, foreign keys, and other dependent objects. The SQL Workbench remains an advanced direct-execution surface; its statements are not intercepted by the guided destructive-confirmation layer. Database work remains synchronous on the UI thread until the worker/actor milestone.

## 7. Non-functional requirements and proposed budgets

These are measurable 1.0 acceptance targets. They must be validated on a documented reference machine and dataset before being advertised.

| Area | 1.0 target | Measurement method |
|---|---|---|
| Startup | Cold start to interactive window ≤ 2.5 s on reference hardware | 10 cold-start runs; report median and p95 |
| UI responsiveness | Common UI interactions p95 ≤ 50 ms while idle | Instrumented UI benchmark |
| Query responsiveness | UI remains interactive while a query runs; cancellation acknowledgement ≤ 250 ms | Long-running recursive query fixture |
| Memory | Idle resident memory ≤ 180 MiB; result preview budget ≤ 64 MiB by default | OS process metrics + large/wide fixture |
| Data safety | Failed edit/import transaction leaves the original database unchanged | Integration tests with injected failures |
| Portability | CI build and smoke test on supported Windows, macOS, and Linux targets | Release matrix |
| Compatibility | Open databases produced by supported SQLite 3.x versions; unsupported features identified rather than silently rewritten | Compatibility fixture suite |
| Privacy | No telemetry or network calls by default | Network-monitoring release test and dependency review |
| Accessibility | Core workflows usable by keyboard; meaningful focus order and contrast | Manual audit + automated checks where available |
| Reliability | No panic for malformed SQL, corrupt fixtures, or common I/O failures | Fuzz, fault-injection, and integration tests |

## 7. Acceptance gates for 1.0

- All P0 workflows are implemented or explicitly removed from the 1.0 promise after product review.
- No query/import/export path can accumulate unbounded in-memory results by default.
- All write workflows are transactional, previewed where destructive, and have tested rollback behavior.
- Read-only mode cannot write to the selected database through any application-supported workflow.
- Database work runs outside the UI thread and supports cancellation where SQLite permits it.
- Cross-platform CI, release packaging, dependency/license review, and security review are green.
- The user guide documents the SQLite version, supported platform matrix, file compatibility, data-safety behavior, and limitations.
