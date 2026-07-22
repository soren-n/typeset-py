use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use ::typeset as native;

mod parser;

/// An unsolved layout tree; built via the module's constructor functions.
#[pyclass(frozen, from_py_object)]
#[derive(Clone)]
struct Layout {
    native: Box<native::Layout>,
}

impl Layout {
    fn wrap(native: Box<native::Layout>) -> Self {
        Layout { native }
    }
}

#[pymethods]
impl Layout {
    fn __repr__(&self) -> String {
        format!("{:?}", self.native)
    }

    /// `left + right`: padded composition, equivalent to `pad(left, right)`.
    fn __add__(&self, other: &Layout) -> Layout {
        Layout::wrap(native::pad(self.native.clone(), other.native.clone()))
    }

    /// `left & right`: unpadded composition, equivalent to `unpad(left, right)`.
    fn __and__(&self, other: &Layout) -> Layout {
        Layout::wrap(native::unpad(self.native.clone(), other.native.clone()))
    }

    /// `left @ right`: forced linebreak, equivalent to `line(left, right)`.
    fn __matmul__(&self, other: &Layout) -> Layout {
        Layout::wrap(native::line(self.native.clone(), other.native.clone()))
    }
}

/// A compiled, render-ready document.
#[pyclass(frozen)]
struct Document {
    native: Box<native::Doc>,
}

#[pymethods]
impl Document {
    fn __repr__(&self) -> String {
        format!("{:?}", self.native)
    }
}

#[pyfunction]
fn null() -> Layout {
    Layout::wrap(native::null())
}

#[pyfunction]
fn text(data: String) -> Layout {
    Layout::wrap(native::text(data))
}

#[pyfunction]
fn fix(layout: Layout) -> Layout {
    Layout::wrap(native::fix(layout.native))
}

#[pyfunction]
fn grp(layout: Layout) -> Layout {
    Layout::wrap(native::grp(layout.native))
}

#[pyfunction]
fn seq(layout: Layout) -> Layout {
    Layout::wrap(native::seq(layout.native))
}

#[pyfunction]
fn nest(layout: Layout) -> Layout {
    Layout::wrap(native::nest(layout.native))
}

#[pyfunction]
fn pack(layout: Layout) -> Layout {
    Layout::wrap(native::pack(layout.native))
}

#[pyfunction]
fn line(left: Layout, right: Layout) -> Layout {
    Layout::wrap(native::line(left.native, right.native))
}

#[pyfunction]
fn pad(left: Layout, right: Layout) -> Layout {
    Layout::wrap(native::pad(left.native, right.native))
}

#[pyfunction]
fn unpad(left: Layout, right: Layout) -> Layout {
    Layout::wrap(native::unpad(left.native, right.native))
}

#[pyfunction]
fn fix_pad(left: Layout, right: Layout) -> Layout {
    Layout::wrap(native::fix_pad(left.native, right.native))
}

#[pyfunction]
fn fix_unpad(left: Layout, right: Layout) -> Layout {
    Layout::wrap(native::fix_unpad(left.native, right.native))
}

#[pyfunction]
fn space() -> Layout {
    Layout::wrap(native::space())
}

#[pyfunction]
fn comma() -> Layout {
    Layout::wrap(native::comma())
}

#[pyfunction]
fn semicolon() -> Layout {
    Layout::wrap(native::semicolon())
}

#[pyfunction]
fn newline() -> Layout {
    Layout::wrap(native::newline())
}

#[pyfunction]
fn blank_line() -> Layout {
    Layout::wrap(native::blank_line())
}

#[pyfunction]
fn join_with(layouts: Vec<Layout>, separator: Layout) -> Layout {
    Layout::wrap(native::join_with(
        layouts.into_iter().map(|layout| layout.native).collect(),
        separator.native,
    ))
}

#[pyfunction]
fn join_with_spaces(layouts: Vec<Layout>) -> Layout {
    Layout::wrap(native::join_with_spaces(
        layouts.into_iter().map(|layout| layout.native).collect(),
    ))
}

#[pyfunction]
fn join_with_commas(layouts: Vec<Layout>) -> Layout {
    Layout::wrap(native::join_with_commas(
        layouts.into_iter().map(|layout| layout.native).collect(),
    ))
}

#[pyfunction]
fn join_with_lines(layouts: Vec<Layout>) -> Layout {
    Layout::wrap(native::join_with_lines(
        layouts.into_iter().map(|layout| layout.native).collect(),
    ))
}

#[pyfunction]
fn parens(layout: Layout) -> Layout {
    Layout::wrap(native::parens(layout.native))
}

#[pyfunction]
fn brackets(layout: Layout) -> Layout {
    Layout::wrap(native::brackets(layout.native))
}

#[pyfunction]
fn braces(layout: Layout) -> Layout {
    Layout::wrap(native::braces(layout.native))
}

#[pyfunction]
fn compile(layout: Layout) -> Document {
    Document {
        native: native::compile(layout.native),
    }
}

#[pyfunction]
fn render(document: &Document, tab: usize, width: usize) -> String {
    native::render(&document.native, tab, width)
}

#[pyfunction]
fn format_layout(layout: Layout, tab: usize, width: usize) -> String {
    native::format_layout(layout.native, tab, width)
}

#[pyfunction]
#[pyo3(signature = (input, *fragments))]
fn parse(input: &str, fragments: &Bound<'_, PyTuple>) -> PyResult<Layout> {
    let fragments = fragments
        .iter()
        .map(|fragment| Ok(fragment.extract::<Layout>()?.native))
        .collect::<PyResult<Vec<_>>>()?;
    parser::parse(input, &fragments)
        .map(Layout::wrap)
        .map_err(PyValueError::new_err)
}

#[pymodule]
fn typeset(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<Layout>()?;
    module.add_class::<Document>()?;
    module.add_function(wrap_pyfunction!(null, module)?)?;
    module.add_function(wrap_pyfunction!(text, module)?)?;
    module.add_function(wrap_pyfunction!(fix, module)?)?;
    module.add_function(wrap_pyfunction!(grp, module)?)?;
    module.add_function(wrap_pyfunction!(seq, module)?)?;
    module.add_function(wrap_pyfunction!(nest, module)?)?;
    module.add_function(wrap_pyfunction!(pack, module)?)?;
    module.add_function(wrap_pyfunction!(line, module)?)?;
    module.add_function(wrap_pyfunction!(pad, module)?)?;
    module.add_function(wrap_pyfunction!(unpad, module)?)?;
    module.add_function(wrap_pyfunction!(fix_pad, module)?)?;
    module.add_function(wrap_pyfunction!(fix_unpad, module)?)?;
    module.add_function(wrap_pyfunction!(space, module)?)?;
    module.add_function(wrap_pyfunction!(comma, module)?)?;
    module.add_function(wrap_pyfunction!(semicolon, module)?)?;
    module.add_function(wrap_pyfunction!(newline, module)?)?;
    module.add_function(wrap_pyfunction!(blank_line, module)?)?;
    module.add_function(wrap_pyfunction!(join_with, module)?)?;
    module.add_function(wrap_pyfunction!(join_with_spaces, module)?)?;
    module.add_function(wrap_pyfunction!(join_with_commas, module)?)?;
    module.add_function(wrap_pyfunction!(join_with_lines, module)?)?;
    module.add_function(wrap_pyfunction!(parens, module)?)?;
    module.add_function(wrap_pyfunction!(brackets, module)?)?;
    module.add_function(wrap_pyfunction!(braces, module)?)?;
    module.add_function(wrap_pyfunction!(compile, module)?)?;
    module.add_function(wrap_pyfunction!(render, module)?)?;
    module.add_function(wrap_pyfunction!(format_layout, module)?)?;
    module.add_function(wrap_pyfunction!(parse, module)?)?;
    Ok(())
}
