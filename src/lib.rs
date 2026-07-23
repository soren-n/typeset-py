use std::sync::Arc;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use ::typeset as native;

mod parser;

/// A deferred layout node.
///
/// Composition does not build the native tree eagerly: it records the child
/// nodes and the native constructor to apply. Because Python `Layout` objects
/// are shared and immutable, eager construction would have to deep-clone every
/// operand's subtree on each `+`/`&`/`@`/`pad(..)`/... call, making expression
/// building quadratic. Storing `Arc` children instead makes every composition
/// O(1); the native tree is materialized once, on demand, at `compile`,
/// `format_layout`, or `repr`.
enum Node {
    /// A leaf whose native form is already built: `text`, `null`, the nullary
    /// helpers, and whole trees returned by `parse`.
    Leaf(Box<native::Layout>),
    Unary(fn(Box<native::Layout>) -> Box<native::Layout>, Arc<Node>),
    Binary(
        fn(Box<native::Layout>, Box<native::Layout>) -> Box<native::Layout>,
        Arc<Node>,
        Arc<Node>,
    ),
    Join(
        fn(Vec<Box<native::Layout>>) -> Box<native::Layout>,
        Vec<Arc<Node>>,
    ),
    JoinWith(
        fn(Vec<Box<native::Layout>>, Box<native::Layout>) -> Box<native::Layout>,
        Vec<Arc<Node>>,
        Arc<Node>,
    ),
}

impl Node {
    /// Build the native layout tree this node describes. Called once per
    /// `compile`/`render`/`repr`; the recursion depth matches the tree depth,
    /// the same shape the native compiler already walks.
    fn materialize(&self) -> Box<native::Layout> {
        match self {
            Node::Leaf(layout) => layout.clone(),
            Node::Unary(build, child) => build(child.materialize()),
            Node::Binary(build, left, right) => build(left.materialize(), right.materialize()),
            Node::Join(build, children) => {
                build(children.iter().map(|child| child.materialize()).collect())
            }
            Node::JoinWith(build, children, separator) => build(
                children.iter().map(|child| child.materialize()).collect(),
                separator.materialize(),
            ),
        }
    }
}

/// An unsolved layout tree; built via the module's constructor functions.
#[pyclass(frozen)]
struct Layout {
    node: Arc<Node>,
}

impl Layout {
    fn new(node: Node) -> Self {
        Layout {
            node: Arc::new(node),
        }
    }

    fn leaf(layout: Box<native::Layout>) -> Self {
        Layout::new(Node::Leaf(layout))
    }

    fn unary(build: fn(Box<native::Layout>) -> Box<native::Layout>, child: &Layout) -> Self {
        Layout::new(Node::Unary(build, child.node.clone()))
    }

    fn binary(
        build: fn(Box<native::Layout>, Box<native::Layout>) -> Box<native::Layout>,
        left: &Layout,
        right: &Layout,
    ) -> Self {
        Layout::new(Node::Binary(build, left.node.clone(), right.node.clone()))
    }
}

fn child_nodes(layouts: &[Bound<'_, Layout>]) -> Vec<Arc<Node>> {
    layouts
        .iter()
        .map(|layout| layout.borrow().node.clone())
        .collect()
}

#[pymethods]
impl Layout {
    fn __repr__(&self) -> String {
        format!("{:?}", self.node.materialize())
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
fn space() -> Layout {
    Layout::leaf(native::space())
}

#[pyfunction]
fn comma() -> Layout {
    Layout::leaf(native::comma())
}

#[pyfunction]
fn semicolon() -> Layout {
    Layout::leaf(native::semicolon())
}

#[pyfunction]
fn newline() -> Layout {
    Layout::leaf(native::newline())
}

#[pyfunction]
fn blank_line() -> Layout {
    Layout::leaf(native::blank_line())
}

#[pyfunction]
fn join_with(layouts: Vec<Bound<'_, Layout>>, separator: &Layout) -> Layout {
    Layout::new(Node::JoinWith(
        native::join_with,
        child_nodes(&layouts),
        separator.node.clone(),
    ))
}

#[pyfunction]
fn join_with_spaces(layouts: Vec<Bound<'_, Layout>>) -> Layout {
    Layout::new(Node::Join(native::join_with_spaces, child_nodes(&layouts)))
}

#[pyfunction]
fn join_with_commas(layouts: Vec<Bound<'_, Layout>>) -> Layout {
    Layout::new(Node::Join(native::join_with_commas, child_nodes(&layouts)))
}

#[pyfunction]
fn join_with_lines(layouts: Vec<Bound<'_, Layout>>) -> Layout {
    Layout::new(Node::Join(native::join_with_lines, child_nodes(&layouts)))
}

#[pyfunction]
fn parens(layout: &Layout) -> Layout {
    Layout::unary(native::parens, layout)
}

#[pyfunction]
fn brackets(layout: &Layout) -> Layout {
    Layout::unary(native::brackets, layout)
}

#[pyfunction]
fn braces(layout: &Layout) -> Layout {
    Layout::unary(native::braces, layout)
}

#[pyfunction]
fn compile(layout: &Layout) -> Document {
    Document {
        native: native::compile(layout.node.materialize()),
    }
}

#[pyfunction]
fn render(document: &Document, tab: usize, width: usize) -> String {
    native::render(&document.native, tab, width)
}

#[pyfunction]
fn format_layout(layout: &Layout, tab: usize, width: usize) -> String {
    native::format_layout(layout.node.materialize(), tab, width)
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
