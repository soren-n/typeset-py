use std::sync::Arc;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use ::typeset as native;

mod node;
mod parser;

use node::Node;

/// An unsolved layout tree; built via the module's constructor functions.
/// The tree is deferred (see `node`): composition records the constructor
/// to apply, and the native layout is built at `compile` or `repr`.
#[pyclass(frozen)]
struct Layout {
    node: Arc<Node>,
}

impl Layout {
    fn leaf(layout: native::Layout) -> Self {
        Layout {
            node: Node::leaf(layout),
        }
    }

    fn unary(build: fn(native::Layout) -> native::Layout, child: &Layout) -> Self {
        Layout {
            node: Node::unary(build, child.node.clone()),
        }
    }

    fn binary(
        build: fn(native::Layout, native::Layout) -> native::Layout,
        left: &Layout,
        right: &Layout,
    ) -> Self {
        Layout {
            node: Node::binary(build, left.node.clone(), right.node.clone()),
        }
    }

    fn join(
        build: fn(Vec<native::Layout>) -> native::Layout,
        layouts: &[Bound<'_, Layout>],
    ) -> Self {
        let children = layouts
            .iter()
            .map(|layout| layout.borrow().node.clone())
            .collect();
        Layout {
            node: Node::join(build, children),
        }
    }
}

#[pymethods]
impl Layout {
    /// The layout in the DSL; `parse` reads it back.
    fn __repr__(&self) -> String {
        self.node.materialize().to_string()
    }

    /// `left + right`: padded composition, equivalent to `pad(left, right)`.
    fn __add__(&self, other: &Layout) -> Layout {
        Layout::binary(native::pad, self, other)
    }

    /// `left & right`: unpadded composition, equivalent to `unpad(left, right)`.
    fn __and__(&self, other: &Layout) -> Layout {
        Layout::binary(native::unpad, self, other)
    }

    /// `left @ right`: forced linebreak, equivalent to `line(left, right)`.
    fn __matmul__(&self, other: &Layout) -> Layout {
        Layout::binary(native::line, self, other)
    }

    /// Compile the layout into a document.
    fn compile(&self) -> Document {
        Document {
            native: self.node.materialize().compile(),
        }
    }
}

/// A compiled, render-ready document.
#[pyclass(frozen)]
struct Document {
    native: native::Doc,
}

#[pymethods]
impl Document {
    /// The document in the DSL: its normal form, which compiles to itself.
    fn __repr__(&self) -> String {
        format!("{:?}", self.native)
    }

    /// Render the document at a tab width and a target line width.
    fn render(&self, tab: usize, width: usize) -> String {
        self.native.render(tab, width)
    }
}

#[pyfunction]
fn null() -> Layout {
    Layout::leaf(native::null())
}

#[pyfunction]
fn text(data: String) -> Layout {
    Layout::leaf(native::text(data))
}

#[pyfunction]
fn fix(layout: &Layout) -> Layout {
    Layout::unary(native::fix, layout)
}

#[pyfunction]
fn grp(layout: &Layout) -> Layout {
    Layout::unary(native::grp, layout)
}

#[pyfunction]
fn seq(layout: &Layout) -> Layout {
    Layout::unary(native::seq, layout)
}

#[pyfunction]
fn nest(layout: &Layout) -> Layout {
    Layout::unary(native::nest, layout)
}

#[pyfunction]
fn pack(layout: &Layout) -> Layout {
    Layout::unary(native::pack, layout)
}

#[pyfunction]
fn line(left: &Layout, right: &Layout) -> Layout {
    Layout::binary(native::line, left, right)
}

#[pyfunction]
fn pad(left: &Layout, right: &Layout) -> Layout {
    Layout::binary(native::pad, left, right)
}

#[pyfunction]
fn unpad(left: &Layout, right: &Layout) -> Layout {
    Layout::binary(native::unpad, left, right)
}

#[pyfunction]
fn fix_pad(left: &Layout, right: &Layout) -> Layout {
    Layout::binary(native::fix_pad, left, right)
}

#[pyfunction]
fn fix_unpad(left: &Layout, right: &Layout) -> Layout {
    Layout::binary(native::fix_unpad, left, right)
}

#[pyfunction]
fn join_with_spaces(layouts: Vec<Bound<'_, Layout>>) -> Layout {
    Layout::join(native::join_with_spaces, &layouts)
}

#[pyfunction]
fn join_with_commas(layouts: Vec<Bound<'_, Layout>>) -> Layout {
    Layout::join(native::join_with_commas, &layouts)
}

#[pyfunction]
fn join_with_lines(layouts: Vec<Bound<'_, Layout>>) -> Layout {
    Layout::join(native::join_with_lines, &layouts)
}

#[pyfunction]
#[pyo3(signature = (input, *fragments))]
fn parse(input: &str, fragments: &Bound<'_, PyTuple>) -> PyResult<Layout> {
    let fragments = fragments
        .iter()
        .map(|fragment| Ok(fragment.extract::<PyRef<'_, Layout>>()?.node.materialize()))
        .collect::<PyResult<Vec<_>>>()?;
    parser::parse(input, &fragments)
        .map(Layout::leaf)
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
    module.add_function(wrap_pyfunction!(join_with_spaces, module)?)?;
    module.add_function(wrap_pyfunction!(join_with_commas, module)?)?;
    module.add_function(wrap_pyfunction!(join_with_lines, module)?)?;
    module.add_function(wrap_pyfunction!(parse, module)?)?;
    Ok(())
}
