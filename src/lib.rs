#![feature(box_patterns)]

use pyo3::prelude::*;
use pyo3::exceptions;
use pyo3::types::PyTuple;

use pest::Parser;
use pest_derive::Parser;
use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;

use ::typeset::{self as native};

#[pyclass]
#[derive(Debug, Clone)]
struct Layout {
  native: Box<native::Layout>
}

#[pyclass]
#[derive(Debug, Clone)]
struct Document {
  native: Box<native::Doc>
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
  Ok(native::print(doc.native))
}

#[pyfunction]
fn compile(layout: Layout) -> PyResult<Document> {
  Ok(Document { native: native::compile(layout.native) })
}

#[pyfunction]
fn render(doc: Document, tab: usize, width: usize) -> PyResult<String> {
  Ok(native::render(doc.native, tab, width))
}

#[derive(Parser)]
#[grammar = "layout.pest"]
pub struct LayoutParser;

lazy_static::lazy_static! {
  static ref PRATT_PARSER: PrattParser<Rule> = {
    use pest::pratt_parser::{Assoc::*, Op};
    PrattParser::new()
      .op(
        Op::infix(Rule::single_line_op, Right) |
        Op::infix(Rule::double_line_op, Right) |
        Op::infix(Rule::unpad_comp_op, Right) |
        Op::infix(Rule::pad_comp_op, Right) |
        Op::infix(Rule::fix_unpad_comp_op, Right) |
        Op::infix(Rule::fix_pad_comp_op, Right)
      )
      .op(
        Op::prefix(Rule::fix_op) |
        Op::prefix(Rule::grp_op) |
        Op::prefix(Rule::seq_op) |
        Op::prefix(Rule::nest_op) |
        Op::prefix(Rule::pack_op)
      )
  };
}

#[derive(Debug)]
enum Syntax {
  Index(usize),
  Text(String),
  Fix(Box<Syntax>),
  Grp(Box<Syntax>),
  Seq(Box<Syntax>),
  Nest(Box<Syntax>),
  Pack(Box<Syntax>),
  SingleLine(Box<Syntax>, Box<Syntax>),
  DoubleLine(Box<Syntax>, Box<Syntax>),
  UnpadComp(Box<Syntax>, Box<Syntax>),
  PadComp(Box<Syntax>, Box<Syntax>),
  FixUnpadComp(Box<Syntax>, Box<Syntax>),
  FixPadComp(Box<Syntax>, Box<Syntax>)
}

fn _parse_syntax(tokens: Pairs<Rule>) -> Result<Box<Syntax>, String> {
  PRATT_PARSER
    .map_primary(|primary| match primary.as_rule() {
      Rule::index =>
        Ok(Box::new(Syntax::Index(primary.as_str().parse::<usize>().unwrap()))),
      Rule::text =>
        Ok(Box::new(Syntax::Text(primary.as_str().to_string()))),
      Rule::expr =>
        _parse_syntax(primary.into_inner()),
      rule =>
        Err(format!("expected atom, found {:?}", rule))
    })
    .map_infix(|left, op, right| match op.as_rule() {
      Rule::single_line_op =>
        Ok(Box::new(Syntax::SingleLine(left?, right?))),
      Rule::double_line_op =>
        Ok(Box::new(Syntax::DoubleLine(left?, right?))),
      Rule::unpad_comp_op =>
        Ok(Box::new(Syntax::UnpadComp(left?, right?))),
      Rule::pad_comp_op =>
        Ok(Box::new(Syntax::PadComp(left?, right?))),
      Rule::fix_unpad_comp_op =>
        Ok(Box::new(Syntax::FixUnpadComp(left?, right?))),
      Rule::fix_pad_comp_op =>
        Ok(Box::new(Syntax::FixPadComp(left?, right?))),
      rule =>
        Err(format!("expected binary operator, found {:?}", rule))
    })
    .map_prefix(|op, syntax| match op.as_rule() {
      Rule::fix_op => Ok(Box::new(Syntax::Fix(syntax?))),
      Rule::grp_op => Ok(Box::new(Syntax::Grp(syntax?))),
      Rule::seq_op => Ok(Box::new(Syntax::Seq(syntax?))),
      Rule::nest_op => Ok(Box::new(Syntax::Nest(syntax?))),
      Rule::pack_op => Ok(Box::new(Syntax::Pack(syntax?))),
      rule =>
        Err(format!("expected unary operator, found {:?}", rule))
    })
    .parse(tokens)
}

fn _interp_syntax(
  syntax: Box<Syntax>,
  args: &Vec<Box<native::Layout>>
) -> Result<Box<native::Layout>, String> {
  match syntax {
    box Syntax::Index(index) => {
      let length = args.len();
      if index < length { Ok(args[index].clone()) } else {
      Err(format!("invalid index {:?}", index)) }
    }
    box Syntax::Text(data) =>
      Ok(native::text(data)),
    box Syntax::Fix(syntax1) => {
      let layout = _interp_syntax(syntax1, args);
      Ok(native::fix(layout?))
    }
    box Syntax::Grp(syntax1) => {
      let layout = _interp_syntax(syntax1, args);
      Ok(native::grp(layout?))
    }
    box Syntax::Seq(syntax1) => {
      let layout = _interp_syntax(syntax1, args);
      Ok(native::seq(layout?))
    }
    box Syntax::Nest(syntax1) => {
      let layout = _interp_syntax(syntax1, args);
      Ok(native::nest(layout?))
    }
    box Syntax::Pack(syntax1) => {
      let layout = _interp_syntax(syntax1, args);
      Ok(native::pack(layout?))
    }
    box Syntax::SingleLine(left, right) => {
      let left1 = _interp_syntax(left, args);
      let right1 = _interp_syntax(right, args);
      Ok(native::line(left1?, right1?))
    }
    box Syntax::DoubleLine(left, right) => {
      let left1 = _interp_syntax(left, args);
      let right1 = _interp_syntax(right, args);
      Ok(native::line(left1?, native::line(native::null(), right1?)))
    }
    box Syntax::UnpadComp(left, right) => {
      let left1 = _interp_syntax(left, args);
      let right1 = _interp_syntax(right, args);
      Ok(native::comp(left1?, right1?, false, false))
    }
    box Syntax::PadComp(left, right) => {
      let left1 = _interp_syntax(left, args);
      let right1 = _interp_syntax(right, args);
      Ok(native::comp(left1?, right1?, true, false))
    }
    box Syntax::FixUnpadComp(left, right) => {
      let left1 = _interp_syntax(left, args);
      let right1 = _interp_syntax(right, args);
      Ok(native::comp(left1?, right1?, false, true))
    }
    box Syntax::FixPadComp(left, right) => {
      let left1 = _interp_syntax(left, args);
      let right1 = _interp_syntax(right, args);
      Ok(native::comp(left1?, right1?, true, true))
    }
  }
}

#[pyfunction]
#[pyo3(signature = (input, *args))]
fn parse(input: String, args: &PyTuple) -> PyResult<Layout> {
  match LayoutParser::parse(Rule::layout, &input) {
    Ok(mut tokens) =>
      Ok(Layout {
        native: _interp_syntax(
          _parse_syntax(tokens.next().unwrap().into_inner())
            .map_err(exceptions::PyValueError::new_err)?,
          &args.iter().map(|layout: &PyAny| {
            layout.extract::<Layout>().unwrap().native
          }).collect()
        ).map_err(exceptions::PyValueError::new_err)?
      }),
    Err(error) => {
      let message = format!("{}", error);
      Err(exceptions::PyValueError::new_err(message))
    }
  }
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