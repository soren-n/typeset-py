# API Design and Python Bindings

## PyO3 Integration Architecture

The project uses PyO3 to create seamless Python bindings for Rust code, following established patterns for performance and safety.

## Core API Design Principles

### 1. Wrapper Pattern
All Python-exposed types wrap Rust native types:

```rust
#[pyclass]
#[derive(Debug, Clone)]
struct Layout {
    native: Box<native::Layout>,  // Rust type wrapped in Box
}

#[pyclass]
#[derive(Debug, Clone)]
struct Document {
    native: Box<native::Doc>,
}
```

**Benefits**:
- Zero-copy data sharing between Python and Rust
- Rust memory safety guarantees
- Efficient cloning (clones Box pointers, not data)

### 2. Error Handling Strategy

All public functions return `PyResult<T>` for proper Python exception handling:

```rust
#[pyfunction]
fn parse(input: String, args: &Bound<'_, PyTuple>) -> PyResult<Layout> {
    let _args: Result<Vec<Box<native::Layout>>, PyErr> = args
        .iter()
        .map(|layout: Bound<'_, PyAny>| -> Result<Box<native::Layout>, PyErr> {
            Ok(layout.extract::<Layout>()?.native)
        })
        .collect();

    Ok(Layout {
        native: parser::parse(input.as_str(), &_args?)
            .map_err(exceptions::PyValueError::new_err)?,  // Convert Rust Result to PyResult
    })
}
```

**Error Conversion Pattern**:
- Rust `Result<T, String>` → Python `ValueError`
- Rust `Result<T, ParseError>` → Python `ValueError`
- PyO3 extraction errors → Python `TypeError`

### 3. Memory Management

**Rust Side**:
- `Box<T>` for heap allocation of large structures
- Automatic cleanup when Python objects are garbage collected
- No manual memory management required

**Python Side**:
```python
layout = typeset.text("hello")    # Rust Box<Layout> created
# ... use layout ...
# layout automatically freed when Python object is GC'd
```

## Function Signatures and Patterns

### Constructor Functions

```rust
#[pyfunction]
fn text(data: String) -> PyResult<Layout> {
    Ok(Layout {
        native: native::text(data)
    })
}

#[pyfunction]
fn comp(left: Layout, right: Layout, pad: bool, fix: bool) -> PyResult<Layout> {
    Ok(Layout {
        native: native::comp(left.native, right.native, pad, fix),
    })
}
```

**Pattern**: Take owned values, return wrapped results

### Variadic Functions (Advanced)

```rust
#[pyfunction]
#[pyo3(signature = (input, *args))]  // Python *args syntax
fn parse(input: String, args: &Bound<'_, PyTuple>) -> PyResult<Layout> {
    // Extract variable arguments from PyTuple
    let _args: Result<Vec<Box<native::Layout>>, PyErr> = args
        .iter()
        .map(|arg| Ok(arg.extract::<Layout>()?.native))
        .collect();
    // ...
}
```

**Usage in Python**:
```python
layout = parse("{0} + {1}", text("hello"), text("world"))
```

### Method Implementation

```rust
#[pymethods]
impl Layout {
    fn __repr__(&self) -> String {
        format!("{}", self.native)  // Delegate to Rust Display trait
    }
}

#[pymethods]
impl Document {
    fn __repr__(&self) -> String {
        format!("{}", self.native)
    }
}
```

**Standard Methods**:
- `__repr__` for debug representation
- Future: `__str__`, `__eq__`, `__hash__` as needed

## Type System Integration

### Python Type Stubs

**File**: `typeset.pyi`

```python
class Layout:
    """A class representing a Layout."""

class Document:
    """A class representing a Document."""

def text(data: str) -> Layout:
    """Construct a text layout."""

def compile(layout: Layout) -> Document:
    """Compile a layout to construct a document."""

def render(document: Document, indent: int, width: int) -> str:
    """Render a document to a string."""
```

**Benefits**:
- IDE autocompletion and type checking
- MyPy static analysis support
- Clear API documentation for users

### Type Safety Patterns

**Rust-Python Boundary**:
```rust
// Safe extraction with error handling
fn extract_layout_arg(arg: &Bound<'_, PyAny>) -> PyResult<Box<native::Layout>> {
    Ok(arg.extract::<Layout>()?.native)  // PyO3 handles type checking
}
```

**Compile-time Safety**:
- Rust type system prevents runtime errors
- PyO3 provides safe Python object interaction
- No unsafe code required for normal operations

## Performance Optimization Patterns

### Zero-Copy Design

```rust
// Avoid unnecessary allocations
#[pyfunction]
fn line(left: Layout, right: Layout) -> PyResult<Layout> {
    Ok(Layout {
        native: native::line(left.native, right.native),  // Move semantics
    })
}
```

### Efficient Cloning

```rust
#[derive(Clone)]  // Clone is cheap (just Box pointer)
struct Layout {
    native: Box<native::Layout>,
}
```

**Cloning Cost**: O(1) - only clones Box pointer, not underlying data

### Memory Layout Optimization

```rust
// Efficient tuple destructuring
let _args: Result<Vec<Box<native::Layout>>, PyErr> = args
    .iter()
    .map(|layout| Ok(layout.extract::<Layout>()?.native))
    .collect();  // Single allocation for Vec
```

## Module Registration

### PyO3 Module Setup

```rust
#[pymodule]
fn typeset(_py: Python, typeset_module: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();  // Enable Rust logging in Python

    // Register classes
    typeset_module.add_class::<Layout>()?;
    typeset_module.add_class::<Document>()?;

    // Register functions
    typeset_module.add_function(wrap_pyfunction!(text, typeset_module)?)?;
    typeset_module.add_function(wrap_pyfunction!(compile, typeset_module)?)?;
    // ...

    Ok(())
}
```

**Registration Pattern**:
1. Initialize logging bridge
2. Register all classes with `add_class`
3. Register all functions with `add_function`
4. Return `Ok(())` for success

## Advanced API Patterns

### Logging Integration

```rust
use pyo3_log;

#[pymodule]
fn typeset(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();  // Bridge Rust logs to Python logging
    // ...
}
```

**Usage**: Rust `log::info!()` appears in Python logging system

### Error Context Enhancement

```rust
.map_err(|e| exceptions::PyValueError::new_err(
    format!("Parse error at position {}: {}", pos, e)
))?
```

**Benefit**: Provides rich error context to Python users

### Future Extensions

**Async Support** (if needed):
```rust
#[pyfunction]
async fn async_render(doc: Document, width: usize) -> PyResult<String> {
    // Async rendering for large documents
}
```

**Custom Exceptions**:
```rust
create_exception!(typeset, ParseError, pyo3::exceptions::PyException);

// Usage:
.map_err(ParseError::new_err)?
```

## API Design Guidelines

### Function Naming
- Use Python conventions: `snake_case` functions
- Keep names concise but descriptive
- Match underlying Rust library where possible

### Parameter Patterns
- Required parameters first
- Optional parameters with defaults where sensible
- Use `*args` for variadic functions sparingly

### Return Values
- Always wrap in `PyResult<T>` for error handling
- Return owned values, not references
- Use standard Python types where possible

### Documentation
- Every public function needs docstring
- Include parameter types and return types
- Provide usage examples in docstrings

## Thread Safety Considerations

### GIL Handling
PyO3 automatically handles the Global Interpreter Lock (GIL):

```rust
#[pyfunction]
fn expensive_computation() -> PyResult<Layout> {
    // PyO3 can release GIL during pure Rust computation
    let result = heavy_rust_computation();  // No Python objects touched
    Ok(wrap_result(result))
}
```

### Shared State
- All types are `Send + Sync` safe
- Immutable data structures avoid synchronization overhead
- No shared mutable state between Python threads

## Testing Patterns

### Integration Tests
```python
def test_basic_functionality():
    layout = typeset.text("hello")
    doc = typeset.compile(layout)
    result = typeset.render(doc, 2, 80)
    assert result == "hello"
```

### Error Testing
```python
def test_parse_error():
    with pytest.raises(ValueError, match="Parse error"):
        typeset.parse("invalid syntax")
```

### Performance Testing
```python
def test_large_document_performance():
    # Test with large layouts to ensure O(n) behavior
    large_layout = create_large_layout(10000)
    doc = typeset.compile(large_layout)
    # Should complete in reasonable time
```
