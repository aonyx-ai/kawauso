# Run all recipes inside the Flox environment
set shell := ["flox", "activate", "--", "sh", "-cu"]

[private]
default:
    @just --list

[private]
pre-commit-checks: pre-commit-fix pre-commit-verify

# Every recipe that rewrites the working tree, in sequence: they overlap each
# other, and nothing may read a file while one of them is writing it. The
# formatters run before the generation so that what is generated is derived
# from formatted sources.
[private]
pre-commit-fix:
    just prettier true
    just format-toml true
    just format-rust true

# Every recipe that only reads, in parallel: the tree has stopped changing, so
# what each of them sees is what the commit will contain.
[private]
pre-commit-verify:
    #!/usr/bin/env -S parallel --shebang --ungroup --jobs {{ num_cpus() }}
    just lint-github-actions
    just lint-markdown
    just lint-rust
    just lint-yaml
    just test-rust

# Build the Rust documentation and force the rustdoc lints to run
build-rustdoc:
    # The cli bin and the aonyx lib document to the same path, so they run apart
    cargo doc --workspace --exclude cli --no-deps --document-private-items
    cargo doc -p cli --no-deps --document-private-items

# Check that Aonyx builds with the latest dependencies
check-latest-deps force="false":
    #!/usr/bin/env bash

    # Abort if git is not clean (but ignore Flox's manifest.lock)
    if [[ {{ force }} != "true" && -n $(git status --porcelain -- ':!.flox/env/manifest.lock') ]]; then
        echo "Git working directory is not clean. Commit or stash changes before running this recipe. Aborting."
        git status --porcelain

        # Print diff on GitHub Actions
        if [ -n "$GITHUB_ACTIONS" ]; then
            git diff
        fi

        exit 1
    fi

    # Update dependencies to latest versions
    cargo update

    # Run tests to ensure the latest versions are compatible
    RUSTFLAGS="-D deprecated" cargo test --all-features --all-targets --locked

# Check that dependencies have compatible open-source licenses and trusted sources
check-dependencies:
    cargo deny check bans licenses sources

# Check that Aonyx builds with the minimal dependencies
check-minimal-deps force="false":
    #!/usr/bin/env bash

    # Abort if git is not clean (but ignore Flox's manifest.lock)
    if [[ {{ force }} != "true" && -n $(git status --porcelain -- ':!.flox/env/manifest.lock') ]]; then
        echo "Git working directory is not clean. Commit or stash changes before running this recipe. Aborting."
        git status --porcelain

        # Print diff on GitHub Actions
        if [ -n "$GITHUB_ACTIONS" ]; then
            git diff
        fi

        exit 1
    fi

    # Install the nightly toolchain if not already installed
    rustup install nightly

    # Update dependencies to minimal versions
    rustup run nightly cargo update -Z direct-minimal-versions

    # Run tests to ensure the minimal versions are compatible
    RUSTFLAGS="-D deprecated" rustup run nightly cargo test --all-features --all-targets --locked

# Check that Aonyx builds with the MSRV
check-msrv:
    #!/usr/bin/env bash

    # Get the MSRV from the Cargo.toml
    MSRV=$(cat Cargo.toml | grep 'rust-version =' | head -n 1 | cut -d '"' -f 2)

    # Install the MSRV toolchain if not already installed
    rustup install "${MSRV}"

    # Run tests using the MSRV
    RUSTFLAGS="-D deprecated" rustup run "${MSRV}" cargo check --all-features --all-targets

# Check that all dependencies in Cargo.toml are used
check-unused-deps:
    #!/usr/bin/env bash

    # Install the nightly toolchain if not already installed
    rustup install nightly

    # Check for unused dependencies
    rustup run nightly cargo udeps

# Format JSON files
format-json fix="false": (prettier fix "{json,json5}")

# Format Markdown files
format-markdown fix="false": (prettier fix "md")

# Format Rust files
format-rust fix="false":
    rustup install -c rustfmt nightly
    rustup run nightly cargo fmt -- --unstable-features {{ if fix != "true" { "--check" } else { "" } }}

# Format TOML files
format-toml fix="false":
    taplo fmt {{ if fix != "true" { "--diff" } else { "" } }}

# Format YAML files
format-yaml fix="false": (prettier fix "{yaml,yml}")

# Lint GitHub Actions workflows
lint-github-actions:
    zizmor -p .

# Lint Markdown files
lint-markdown:
    markdownlint **/*.md

# Lint Rust files
lint-rust:
    cargo clippy --all-targets --all-features -- -D warnings

# Lint TOML files
lint-toml:
    taplo check

# Lint YAML files
lint-yaml:
    yamllint .

# Run a subset of checks as pre-commit hooks
pre-commit:
    @just pre-commit-checks

# Auto-format files with prettier
prettier fix="false" extension="*":
    prettier {{ if fix == "true" { "--write" } else { "--list-different" } }} --ignore-unknown "**/*.{{ extension }}"

# Run the tests
test-rust:
    cargo llvm-cov nextest {{ if env("GITHUB_ACTIONS", "") != "" { "--lcov --output-path target/lcov.info" } else { "" } }} --all-features --all-targets
