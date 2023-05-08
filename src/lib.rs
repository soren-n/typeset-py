#![feature(box_patterns)]

use pyo3::prelude::*;
use pyo3::exceptions;
use pyo3::types::PyTuple;

use ::typeset::{self as native};

mod parser;

#[pyclass]
#[derive(Debug, Clone)]
struct Layout {
  native: Box<native::Layout>
}

#[pymethods]
impl Layout {
  fn __repr__(&self) -> String {
    format!("{}", self.native)
  }
}

#[pyclass]
#[derive(Debug, Clone)]
struct Document {
  native: Box<native::Doc>
}

#[pymethods]
impl Document {
  fn __repr__(&self) -> String {
    format!("{}", self.native)
  }
}

#[pyfunction]
fn null() -> PyResult<Layout> {
  Ok(Layout { native: native::null() })
}

#[pyfunction]
fn text(data: String) -> PyResult<Layout> {
  Ok(Layout { native: native::text(data) })
}

#[pyfunction]
fn fix(layout: Layout) -> PyResult<Layout> {
  Ok(Layout { native: native::fix(layout.native) })
}

#[pyfunction]
fn grp(layout: Layout) -> PyResult<Layout> {
  Ok(Layout { native: native::grp(layout.native) })
}

#[pyfunction]
fn seq(layout: Layout) -> PyResult<Layout> {
  Ok(Layout { native: native::seq(layout.native) })
}

#[pyfunction]
fn nest(layout: Layout) -> PyResult<Layout> {
  Ok(Layout { native: native::nest(layout.native) })
}

#[pyfunction]
fn pack(layout: Layout) -> PyResult<Layout> {
  Ok(Layout { native: native::pack(layout.native) })
}

#[pyfunction]
fn line(left: Layout, right: Layout) -> PyResult<Layout> {
  Ok(Layout { native: native::line(left.native, right.native) })
}

#[pyfunction]
fn comp(left: Layout, right: Layout, pad: bool, fix: bool) -> PyResult<Layout> {
  Ok(Layout { native: native::comp(left.native, right.native, pad, fix) })
}

#[pyfunction]
fn print(doc: Document) -> PyResult<String> {
  Ok(format!("{}", doc.native))
}

#[pyfunction]
fn compile(layout: Layout) -> PyResult<Document> {
  Ok(Document { native: native::compile(layout.native) })
}

#[pyfunction]
fn render(doc: Document, tab: usize, width: usize) -> PyResult<String> {
  Ok(native::render(doc.native, tab, width))
}

#[pyfunction]
#[pyo3(signature = (input, *args))]
fn parse(input: String, args: &PyTuple) -> PyResult<Layout> {
  let _args: Result<Vec<Box<native::Layout>>, PyErr> =
    args.iter().map(|layout: &PyAny| -> Result<Box<native::Layout>, PyErr> {
      Ok(layout.extract::<Layout>()?.native)
    }).collect();
  Ok(Layout {
    native: parser::parse(
      input.as_str(),
      &_args?
    ).map_err(exceptions::PyValueError::new_err)?
  })
}

#[pymodule]
fn typeset(_py: Python, typeset_module: &PyModule) -> PyResult<()> {
  pyo3_log::init();
  typeset_module.add_class::<Layout>()?;
  typeset_module.add_class::<Document>()?;
  let _null = wrap_pyfunction!(null, typeset_module)?;
  let _text = wrap_pyfunction!(text, typeset_module)?;
  let _fix = wrap_pyfunction!(fix, typeset_module)?;
  let _grp = wrap_pyfunction!(grp, typeset_module)?;
  let _seq = wrap_pyfunction!(seq, typeset_module)?;
  let _nest = wrap_pyfunction!(nest, typeset_module)?;
  let _pack = wrap_pyfunction!(pack, typeset_module)?;
  let _line = wrap_pyfunction!(line, typeset_module)?;
  let _comp = wrap_pyfunction!(comp, typeset_module)?;
  let _print = wrap_pyfunction!(print, typeset_module)?;
  let _compile = wrap_pyfunction!(compile, typeset_module)?;
  let _render = wrap_pyfunction!(render, typeset_module)?;
  let _parse = wrap_pyfunction!(parse, typeset_module)?;
  typeset_module.add_function(_null)?;
  typeset_module.add_function(_text)?;
  typeset_module.add_function(_fix)?;
  typeset_module.add_function(_grp)?;
  typeset_module.add_function(_seq)?;
  typeset_module.add_function(_nest)?;
  typeset_module.add_function(_pack)?;
  typeset_module.add_function(_line)?;
  typeset_module.add_function(_comp)?;
  typeset_module.add_function(_print)?;
  typeset_module.add_function(_compile)?;
  typeset_module.add_function(_render)?;
  typeset_module.add_function(_parse)?;
  Ok(())
}