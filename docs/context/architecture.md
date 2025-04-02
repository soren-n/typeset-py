# Project Architecture

## Overview

typeset-py is a Rust-based Python extension that provides a Domain Specific Language (DSL) for pretty printing. The project uses PyO3 to create Python bindings for a high-performance Rust library.

## Core Architecture

### Two-Phase Compilation Model

```
Layout (AST) → compile() → Document (Optimized) → render() → String (Output)
```

1. **Layout Phase**: User constructs layout trees using constructors or DSL
2. **Compilation Phase**: Layout is optimized into an immutable Document
3. **Rendering Phase**: Document is rendered with specific width/indent parameters

### Component Structure

```
src/lib.rs           # PyO3 bindings and Python API
src/parser.rs        # DSL parser implementation
src/layout.pest      # Pest grammar definition
typeset.pyi          # Python type stubs
```

## Design Patterns

### Wrapper Pattern (PyO3 Integration)

```rust
#[pyclass]
struct Layout {
    native: Box<native::Layout>,  // Wraps Rust native type
}

#[pyclass]
struct Document {
    native: Box<native::Doc>,     // Wraps Rust native type
}
```

**Key Principles:**
- Python objects wrap Rust native types in `Box<T>`
- All operations delegate to native Rust implementations
- Error handling uses `PyResult<T>` for proper Python exception propagation

### Builder Pattern (Layout Construction)

Layout construction follows a functional builder pattern:

```python
# Compositional API
layout = comp(
    text("function"),
    nest(comp(
        text("("),
        seq(args),
        text(")")
    ))
)

# DSL Alternative
layout = parse('fix ("function" + nest ("(" + seq args + ")"))', args)
```

### Pratt Parser (Expression Parsing)

The DSL uses a Pratt parser for handling operator precedence:

```rust
static ref PRATT_PARSER: PrattParser<Rule> = {
    PrattParser::new()
        .op(Op::infix(Rule::pad_comp_op, Right))      // + (padded composition)
        .op(Op::infix(Rule::unpad_comp_op, Right))    // & (unpadded composition)
        .op(Op::prefix(Rule::fix_op))                 // fix (fixed layout)
        // ...
};
```

## Memory Management

### Rust Side
- Uses `Box<T>` for heap allocation of layout trees
- Leverages Rust's ownership system for memory safety
- No manual memory management required

### Python Side
- PyO3 handles Python object lifecycle
- Rust objects automatically cleaned up when Python objects are garbage collected
- No memory leaks between language boundaries

## Error Handling Strategy

### Rust → Python Error Propagation

```rust
fn parse(input: String, args: &Bound<'_, PyTuple>) -> PyResult<Layout> {
    // Rust Result<T, E> converts to PyResult<T>
    Ok(Layout {
        native: parser::parse(input.as_str(), &_args?)
            .map_err(exceptions::PyValueError::new_err)?,  // Convert to Python exception
    })
}
```

### Error Types
- **Parse Errors**: Invalid DSL syntax → `PyValueError`
- **Index Errors**: Invalid template parameters → `PyValueError`
- **Runtime Errors**: Rust panics become Python exceptions

## Performance Characteristics

### Compilation
- Layout → Document compilation is O(n) where n = layout tree size
- Document is immutable and optimized for repeated rendering

### Rendering
- Document → String rendering is O(m) where m = output text length
- Greedy algorithm for line breaking decisions
- No backtracking or complex optimization

### Memory Usage
- Layout trees: O(n) where n = number of constructors
- Documents: O(n) optimized representation
- Rendering: O(m) where m = output length

## Thread Safety

### Rust Components
- All native types are `Send + Sync`
- Immutable data structures after compilation
- No shared mutable state

### Python Integration
- PyO3 handles GIL (Global Interpreter Lock) automatically
- Safe to use from multiple Python threads
- Rust computations can release GIL for better concurrency

## Extension Points

### Adding New Layout Constructors

1. Add to native typeset library dependency
2. Wrap in PyO3 function in `src/lib.rs`
3. Add to module registration in `typeset()` function
4. Update `typeset.pyi` with type signature

### Extending DSL Grammar

1. Add grammar rules to `src/layout.pest`
2. Extend `Syntax` enum in `src/parser.rs`
3. Add parsing logic in `_parse_syntax()`
4. Add interpretation logic in `_interp_syntax()`

### Performance Optimizations

The two-phase model enables several optimizations:
- **Compile once, render many**: Reuse Documents for different widths
- **Lazy evaluation**: Only compute layouts that fit
- **Memory sharing**: Immutable Documents can be shared safely
