run *args:
    cargo run --locked --bin firecrawl -- {{args}}

run-ai *args:
    AGENT=1 cargo run --locked --bin firecrawl -- {{args}}

build:
    cargo build --locked --release --bin firecrawl
    mkdir -p bin
    cp target/release/firecrawl bin/firecrawl

test:
    cargo test --locked --all-targets

lint:
    rustfmt --check --edition 2024 --config skip_children=true src/args.rs src/options.rs src/output.rs src/runtime.rs src/lib.rs src/main.rs tests/sdk.rs tests/workflows.rs tests/documentation.rs
    cargo clippy --locked --all-targets -- -D warnings

coverage:
    cargo llvm-cov --locked --all-targets --ignore-filename-regex '/vendor/' --fail-under-lines 90

doc:
    RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps
    cargo test --locked --doc

check: lint test coverage doc
