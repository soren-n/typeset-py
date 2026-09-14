# Architecture

typeset-py is a thin PyO3 binding over the [typeset](https://docs.rs/typeset)
Rust crate. All layout semantics (solving, compilation, rendering) and the
DSL grammar live upstream; this repository owns only the Python surface and
the DSL front end that splices Python fragments into a script.

## Two-phase model

```
Layout (tree) --.compile()--> Document (immutable) --.render(tab, width)--> str
```

Compile once, render at any number of widths. There is no one-shot helper:
`layout.compile().render(tab, width)` is the one-shot form.

## Deferred construction

The upstream constructors consume `typeset::Layout` by value, but Python
`Layout` objects are shared and immutable, so a naive binding would clone
each operand's arena on every `+`/`&`/`pad(..)`/... call — making expression
building quadratic. Instead, `Layout` wraps an `Arc<Node>` that records the
child nodes and the upstream constructor to apply. Composition is therefore
O(1) (an `Arc` bump), and the native `typeset::Layout` is materialized
exactly once, on demand, at `compile` or `repr`.

Both walks over the deferred tree are iterative, like the upstream
pipeline: materialize is a post-order walk with an explicit stack, and
`Node`'s `Drop` hands children to a worklist, unwrapping each `Arc` it
owned alone. A layout as deep as Python cares to build (a `reduce` over
a hundred thousand words, say) compiles, prints and frees in constant
native stack, with depth costing heap.

## Components

- `src/lib.rs` - the entire Python API: two frozen pyclasses (`Layout`,
  `Document`), one `#[pyfunction]` per upstream constructor, and the module
  init. `Document` wraps `typeset::Doc`; `Layout` wraps an `Arc<Node>`.
  `repr` of either type is its DSL form, which upstream's `Display`/`Debug`
  print.
- `src/node.rs` - the deferred tree (see
  [Deferred construction](#deferred-construction)): a `Node` is one upstream
  constructor over its children, with the iterative materialize and drop.
  Nothing else in the binding is more than conversion.
- `src/parser.rs` - the runtime DSL front end. Upstream's `FromStr` has no
  variables, so this module tokenizes the script itself (the same tokens
  upstream produces, plus `{i}` as a `Token::Var`) and feeds the hidden
  `typeset::dsl::grammar::parse_tokens` with a `Build` implementation that
  maps a variable to its fragment. The grammar itself is never restated
  here: precedence, associativity, `@@` desugaring and error messages are
  upstream's. Errors are `String`s carrying a byte offset, surfaced to
  Python as `ValueError`.
- `typeset.pyi` - handwritten stubs; keyword names must match the Rust
  parameter names exactly, which the pytest suite asserts.

## Conventions

- Layout values are immutable and shared by `Arc`; composition is deferred
  (see above), so Python-side reuse of a layout (e.g. a shared separator) is
  expected, safe, and cheap. A reused subtree is materialized once per
  enclosing `compile`.
- The Python API mirrors the upstream crate's constructors one to one:
  nothing upstream removed survives here as a shim.
- No fallback paths; user-facing failures are typed Python exceptions
  (`ValueError` for parse errors, `TypeError` for wrong argument types).
- The version has a single source of truth: `Cargo.toml`, exported to
  Python as `typeset.__version__` at build time.

## Testing

- `cargo test` runs the front-end and deferred-tree unit tests (the crate
  builds an rlib alongside the cdylib for exactly this purpose; doctests are
  disabled due to the module/upstream crate name collision). Tree shapes are
  asserted through the DSL that upstream prints, so a test reads as the
  script it parsed. The depth tests run a million levels deep on the test
  thread's small stack, so any recursion over the tree fails them.
- `pytest` exercises the built extension end to end; build it first with
  `maturin develop`.
- On macOS, if `cargo test` fails to link libpython, point pyo3 at the
  project venv: `PYO3_PYTHON=$PWD/.venv/bin/python cargo test`.
