$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

cargo fmt --all
if ($LASTEXITCODE -ne 0) { throw "cargo fmt failed" }

cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "cargo clippy failed" }

cargo test --all-targets
if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }

cargo run
if ($LASTEXITCODE -ne 0) { throw "cargo run failed" }
