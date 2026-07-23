# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

typeset-py is a Rust-based Python extension module that provides a DSL for defining source code pretty printers. It uses PyO3 to bind the upstream [typeset](https://docs.rs/typeset) crate; layout semantics live upstream, this repository owns the Python surface and the runtime DSL parser.

## Quick Start

**Package Manager**: This project uses `uv` for Python package management.

```bash
# Setup development environment
uv venv && uv pip install maturin pytest pre-commit

# Build and test
maturin develop
pytest
```

## Essential Commands

```bash
# Development workflow
maturin develop              # Build extension into the venv
cargo test                   # Rust parser unit tests
pytest                       # Python end-to-end tests (build first)
cargo fmt && cargo clippy    # Format and lint Rust code
pre-commit run --all-files   # Run all quality checks

# If cargo test fails to link libpython on macOS:
PYO3_PYTHON=$PWD/.venv/bin/python cargo test
```

## Context Documentation

- **[architecture.md](docs/context/architecture.md)** - binding structure, two-phase compile/render model, conventions, testing
- **[dsl-grammar.md](docs/context/dsl-grammar.md)** - DSL specification; the parser unit tests are its executable form

## Key Files

- `src/lib.rs` - PyO3 bindings and Python API
- `src/parser.rs` - DSL parser (Pest + Pratt) and its unit tests
- `src/layout.pest` - grammar definition for DSL syntax
- `typeset.pyi` - Python type stubs; keyword names must match Rust parameter names
- `tests/test_typeset.py` - end-to-end tests, doubling as API examples

## Dependencies

- **Rust**: pyo3 0.29 (`abi3-py310`: one stable-ABI wheel per platform covers
  CPython 3.10+), typeset 4.1, pest 2.8.7; edition 2024, MSRV 1.89
- **Python**: >=3.10, maturin >=1.9 for building

## DSL Quick Reference

```python
parse('"hello" + "world"')     # Padded composition: "hello world"
parse('"a" & "b"')             # Unpadded: "ab"
parse('"line1" @ "line2"')     # Line break
parse('"a" @@ "b"')            # Double line break (one blank line)
parse('fix ("a" + "b")')       # Fixed (no breaking)
parse('nest {0}', content)     # Nested indentation
```

The same compositions exist as Python operators on Layout: `+` (padded), `&` (unpadded), `@` (line break).

## Conventions

- No backwards-compatibility shims and no fallback code paths; unreachable states are `unreachable!()`, user errors are typed Python exceptions.
- Releases are cut by semantic-release from conventional commits; the version's single source of truth is `Cargo.toml` (exported as `typeset.__version__`).
