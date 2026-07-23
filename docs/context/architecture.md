# Architecture

typeset-py is a thin PyO3 binding over the [typeset](https://docs.rs/typeset)
Rust crate, plus a runtime parser for the layout DSL. All layout semantics
(solving, compilation, rendering) live upstream; this repository owns only
the Python surface and the DSL parser.

## Two-phase model

```
Layout (tree) --compile()--> Document (immutable) --render(tab, width)--> str
```

Compile once, render at any number of widths. `format_layout` is the
one-shot composition of the two.

## Deferred construction

The upstream constructors consume `Box<typeset::Layout>` by value, but Python
`Layout` objects are shared and immutable, so a naive binding would deep-clone
each operand's subtree on every `+`/`&`/`pad(..)`/... call — making expression
building quadratic. Instead, `Layout` wraps an `Arc<Node>` that records the
child nodes and the upstream constructor to apply. Composition is therefore
O(1) (an `Arc` bump), and the native `typeset::Layout` tree is materialized
exactly once, on demand, at `compile`, `format_layout`, or `repr`. Materialize
recursion matches the tree depth the upstream compiler already walks.

## Components

- `src/lib.rs` - the entire Python API: two frozen pyclasses (`Layout`,
  `Document`), one `#[pyfunction]` per upstream constructor, and the module
  init. `Document` wraps `Box<typeset::Doc>`; `Layout` wraps a deferred
  `Arc<Node>` tree (see [Deferred construction](#deferred-construction)). The
  only logic beyond conversion is that deferral; each `Node` variant still maps
  to a single upstream constructor.
- `src/parser.rs` - runtime DSL parser. A Pest grammar produces a token
  stream that a Pratt parser folds directly into `typeset::Layout` values;
  `{i}` placeholders are substituted from the fragment arguments during the
  fold. Errors are `String`s, surfaced to Python as `ValueError`.
- `src/layout.pest` - the grammar. Ordered choices are longest-first
  (PEG choice commits, so `@` must not shadow `@@`).
- `typeset.pyi` - handwritten stubs; keyword names must match the Rust
  parameter names exactly, which the pytest suite asserts.

## Conventions

- Layout values are immutable and shared by `Arc`; composition is deferred
  (see above), so Python-side reuse of a layout (e.g. a shared separator) is
  expected, safe, and cheap. A reused subtree is materialized once per
  enclosing `compile`.
- Match arms that the grammar makes unreachable are `unreachable!()`, not
  error returns: a grammar/code disagreement is a bug and must fail loudly.
- No fallback paths; user-facing failures are typed Python exceptions
  (`ValueError` for parse errors, `TypeError` for wrong argument types).
- The version has a single source of truth: `Cargo.toml`, exported to
  Python as `typeset.__version__` at build time.

## Testing

- `cargo test` runs parser unit tests (the crate builds an rlib alongside
  the cdylib for exactly this purpose; doctests are disabled due to the
  module/upstream crate name collision).
- `pytest` exercises the built extension end to end; build it first with
  `maturin develop`.
- On macOS, if `cargo test` fails to link libpython, point pyo3 at the
  project venv: `PYO3_PYTHON=$PWD/.venv/bin/python cargo test`.
