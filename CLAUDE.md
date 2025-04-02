# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

typeset-py is a Rust-based Python extension module that provides a DSL for defining source code pretty printers. It uses PyO3 to create Python bindings for high-performance Rust code.

## Quick Start

**Package Manager**: This project uses `uv` for Python package management.

```bash
# Setup development environment
uv venv && source .venv/bin/activate
uv pip install pre-commit && pre-commit install

# Build and test
maturin develop
python -c "import typeset; print(typeset.render(typeset.compile(typeset.text('hello')), 2, 80))"
```

## Essential Commands

```bash
# Development workflow
maturin develop              # Build extension for development
cargo fmt && cargo clippy   # Format and lint Rust code
pre-commit run --all-files  # Run all quality checks

# Release workflow
maturin build --release     # Build optimized wheel
cargo test                  # Run Rust tests
```

## Context Documentation

For deep technical understanding, see the context documents in `docs/context/`:

- **[architecture.md](docs/context/architecture.md)** - Project architecture, design patterns, two-phase compilation model, PyO3 integration patterns
- **[dsl-grammar.md](docs/context/dsl-grammar.md)** - Complete DSL specification, Pest grammar, operator precedence, parsing implementation
- **[build-system.md](docs/context/build-system.md)** - Maturin build system, dependency management, CI/CD pipeline, cross-platform builds
- **[api-design.md](docs/context/api-design.md)** - PyO3 binding patterns, error handling, memory management, type system integration
- **[development-workflow.md](docs/context/development-workflow.md)** - Pre-commit hooks, testing strategy, debugging, performance monitoring

## Key Files

- `src/lib.rs` - PyO3 bindings and Python API (166 lines)
- `src/parser.rs` - DSL parser using Pest grammar (192 lines)
- `src/layout.pest` - Grammar definition for DSL syntax
- `typeset.pyi` - Python type stubs for IDE support

## Dependencies

- **Rust**: pyo3 0.25.1, typeset 3.1.0, pest 2.8.1
- **Python**: >=3.8, maturin >=1.8 for building
- **Development**: pre-commit, mypy, cargo audit

## DSL Quick Reference

```python
# Basic syntax
parse('"hello" + "world"')     # Padded composition: "hello world"
parse('"a" & "b"')             # Unpadded: "ab"
parse('"line1" @ "line2"')     # Line break
parse('fix ("a" + "b")')       # Fixed (no breaking)
parse('nest {0}', content)     # Nested indentation
```

See [dsl-grammar.md](docs/context/dsl-grammar.md) for complete specification.
