default: check

# Format, lint, and test
check:
    cargo fmt --all --check
    cargo clippy --all-targets -- -D warnings
    cargo test

fmt:
    cargo fmt --all

run *args:
    cargo run -q -p bluepencil -- {{args}}

# Run the report against the sample novel
demo:
    cd examples/novel && cargo run -q -p bluepencil -- report

install:
    cargo install --path crates/bluepencil

docs:
    mdbook serve docs

audit:
    cargo deny check
