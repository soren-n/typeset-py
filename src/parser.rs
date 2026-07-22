use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;
use pest::Parser;
use pest_derive::Parser;

use typeset::{comp, fix, grp, line, nest, null, pack, seq, text, Break, Layout, Pad};

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
    Null,
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
    FixPadComp(Box<Syntax>, Box<Syntax>),
}

#[doc(hidden)]
pub fn parse(input: &str, args: &[Box<Layout>]) -> Result<Box<Layout>, String> {
    fn _parse_syntax(tokens: Pairs<Rule>) -> Result<Box<Syntax>, String> {
        PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::null => Ok(Box::new(Syntax::Null)),
                Rule::index => Ok(Box::new(Syntax::Index(
                    primary.as_str().parse::<usize>().unwrap(),
                ))),
                Rule::text => primary
                    .into_inner()
                    .try_fold(String::new(), |mut result, part| match part.as_rule() {
                        Rule::raw_string => {
                            result.push_str(part.as_str());
                            Ok(result)
                        }
                        Rule::escaped_string => match &part.as_str()[1..] {
                            "n" => {
                                result.push('\n');
                                Ok(result)
                            }
                            "r" => {
                                result.push('\r');
                                Ok(result)
                            }
                            "t" => {
                                result.push('\t');
                                Ok(result)
                            }
                            "\\" => {
                                result.push('\\');
                                Ok(result)
                            }
                            "0" => {
                                result.push('\0');
                                Ok(result)
                            }
                            "\"" => {
                                result.push('\"');
                                Ok(result)
                            }
                            "'" => {
                                result.push('\'');
                                Ok(result)
                            }
                            char => Err(format!("Unexpected escaped character: \\{char:?}")),
                        },
                        _ => Err(format!("Unexpected token: {part:?}")),
                    })
                    .map(|result| Box::new(Syntax::Text(result))),
                Rule::expr => _parse_syntax(primary.into_inner()),
                rule => Err(format!("expected atom, found {:?}", rule)),
            })
            .map_infix(|left, op, right| match op.as_rule() {
                Rule::single_line_op => Ok(Box::new(Syntax::SingleLine(left?, right?))),
                Rule::double_line_op => Ok(Box::new(Syntax::DoubleLine(left?, right?))),
                Rule::unpad_comp_op => Ok(Box::new(Syntax::UnpadComp(left?, right?))),
                Rule::pad_comp_op => Ok(Box::new(Syntax::PadComp(left?, right?))),
                Rule::fix_unpad_comp_op => Ok(Box::new(Syntax::FixUnpadComp(left?, right?))),
                Rule::fix_pad_comp_op => Ok(Box::new(Syntax::FixPadComp(left?, right?))),
                rule => Err(format!("expected binary operator, found {:?}", rule)),
            })
            .map_prefix(|op, syntax| match op.as_rule() {
                Rule::fix_op => Ok(Box::new(Syntax::Fix(syntax?))),
                Rule::grp_op => Ok(Box::new(Syntax::Grp(syntax?))),
                Rule::seq_op => Ok(Box::new(Syntax::Seq(syntax?))),
                Rule::nest_op => Ok(Box::new(Syntax::Nest(syntax?))),
                Rule::pack_op => Ok(Box::new(Syntax::Pack(syntax?))),
                rule => Err(format!("expected unary operator, found {:?}", rule)),
            })
            .parse(tokens)
    }
    #[allow(clippy::boxed_local)]
    fn _interp_syntax(syntax: Box<Syntax>, args: &[Box<Layout>]) -> Result<Box<Layout>, String> {
        match *syntax {
            Syntax::Null => Ok(null()),
            Syntax::Index(index) => {
                let length = args.len();
                if index < length {
                    Ok(args[index].clone())
                } else {
                    Err(format!("invalid index {:?}", index))
                }
            }
            Syntax::Text(data) => Ok(text(data)),
            Syntax::Fix(syntax1) => {
                let layout = _interp_syntax(syntax1, args);
                Ok(fix(layout?))
            }
            Syntax::Grp(syntax1) => {
                let layout = _interp_syntax(syntax1, args);
                Ok(grp(layout?))
            }
            Syntax::Seq(syntax1) => {
                let layout = _interp_syntax(syntax1, args);
                Ok(seq(layout?))
            }
            Syntax::Nest(syntax1) => {
                let layout = _interp_syntax(syntax1, args);
                Ok(nest(layout?))
            }
            Syntax::Pack(syntax1) => {
                let layout = _interp_syntax(syntax1, args);
                Ok(pack(layout?))
            }
            Syntax::SingleLine(left, right) => {
                let left1 = _interp_syntax(left, args);
                let right1 = _interp_syntax(right, args);
                Ok(line(left1?, right1?))
            }
            Syntax::DoubleLine(left, right) => {
                let left1 = _interp_syntax(left, args);
                let right1 = _interp_syntax(right, args);
                Ok(line(left1?, line(null(), right1?)))
            }
            Syntax::UnpadComp(left, right) => {
                let left1 = _interp_syntax(left, args);
                let right1 = _interp_syntax(right, args);
                Ok(comp(left1?, right1?, Pad::Unpadded, Break::Breakable))
            }
            Syntax::PadComp(left, right) => {
                let left1 = _interp_syntax(left, args);
                let right1 = _interp_syntax(right, args);
                Ok(comp(left1?, right1?, Pad::Padded, Break::Breakable))
            }
            Syntax::FixUnpadComp(left, right) => {
                let left1 = _interp_syntax(left, args);
                let right1 = _interp_syntax(right, args);
                Ok(comp(left1?, right1?, Pad::Unpadded, Break::Fixed))
            }
            Syntax::FixPadComp(left, right) => {
                let left1 = _interp_syntax(left, args);
                let right1 = _interp_syntax(right, args);
                Ok(comp(left1?, right1?, Pad::Padded, Break::Fixed))
            }
        }
    }
    match LayoutParser::parse(Rule::layout, input) {
        Ok(mut tokens) => _interp_syntax(_parse_syntax(tokens.next().unwrap().into_inner())?, args),
        Err(error) => Err(format!("{}", error)),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use typeset::text;

    fn parsed(input: &str) -> String {
        format!("{:?}", parse(input, &[]).unwrap())
    }

    fn parse_err(input: &str) -> String {
        parse(input, &[]).unwrap_err()
    }

    #[test]
    fn null_literal() {
        assert_eq!(parsed("null"), "Null");
    }

    #[test]
    fn text_literal() {
        assert_eq!(parsed(r#""hello""#), r#"Text("hello")"#);
    }

    #[test]
    fn empty_text_literal() {
        assert_eq!(parsed(r#""""#), r#"Text("")"#);
    }

    #[test]
    fn escape_sequences() {
        assert_eq!(
            parsed(r#""a\nb\rc\td\\e\0f\"g\'h""#),
            "Text(\"a\\nb\\rc\\td\\\\e\\0f\\\"g'h\")"
        );
    }

    #[test]
    fn invalid_escape_is_rejected() {
        parse_err(r#""\q""#);
    }

    #[test]
    fn single_line_composition() {
        assert_eq!(parsed(r#""a" @ "b""#), r#"Line(Text("a"), Text("b"))"#);
    }

    #[test]
    fn unpadded_composition() {
        assert_eq!(
            parsed(r#""a" & "b""#),
            r#"Comp(Text("a"), Text("b"), Attr { pad: Unpadded, brk: Breakable })"#
        );
    }

    #[test]
    fn padded_composition() {
        assert_eq!(
            parsed(r#""a" + "b""#),
            r#"Comp(Text("a"), Text("b"), Attr { pad: Padded, brk: Breakable })"#
        );
    }

    #[test]
    fn fixed_unpadded_composition() {
        assert_eq!(
            parsed(r#""a" !& "b""#),
            r#"Comp(Text("a"), Text("b"), Attr { pad: Unpadded, brk: Fixed })"#
        );
    }

    #[test]
    fn fixed_padded_composition() {
        assert_eq!(
            parsed(r#""a" !+ "b""#),
            r#"Comp(Text("a"), Text("b"), Attr { pad: Padded, brk: Fixed })"#
        );
    }

    #[test]
    fn binary_operators_are_right_associative() {
        assert_eq!(
            parsed(r#""a" + "b" + "c""#),
            concat!(
                r#"Comp(Text("a"), Comp(Text("b"), Text("c"), "#,
                r#"Attr { pad: Padded, brk: Breakable }), "#,
                r#"Attr { pad: Padded, brk: Breakable })"#
            )
        );
    }

    #[test]
    fn parentheses_override_associativity() {
        assert_eq!(
            parsed(r#"("a" + "b") + "c""#),
            concat!(
                r#"Comp(Comp(Text("a"), Text("b"), "#,
                r#"Attr { pad: Padded, brk: Breakable }), Text("c"), "#,
                r#"Attr { pad: Padded, brk: Breakable })"#
            )
        );
    }

    #[test]
    fn unary_operators() {
        assert_eq!(parsed(r#"fix "a""#), r#"Fix(Text("a"))"#);
        assert_eq!(parsed(r#"grp "a""#), r#"Grp(Text("a"))"#);
        assert_eq!(parsed(r#"seq "a""#), r#"Seq(Text("a"))"#);
        assert_eq!(parsed(r#"nest "a""#), r#"Nest(Text("a"))"#);
        assert_eq!(parsed(r#"pack "a""#), r#"Pack(Text("a"))"#);
    }

    #[test]
    fn unary_operator_over_parenthesized_expression() {
        assert_eq!(
            parsed(r#"fix ("a" + "b")"#),
            r#"Fix(Comp(Text("a"), Text("b"), Attr { pad: Padded, brk: Breakable }))"#
        );
    }

    #[test]
    fn unary_binds_tighter_than_binary() {
        assert_eq!(
            parsed(r#"fix "a" + "b""#),
            r#"Comp(Fix(Text("a")), Text("b"), Attr { pad: Padded, brk: Breakable })"#
        );
    }

    #[test]
    fn index_substitution() {
        let args = [text("left"), text("right")];
        assert_eq!(
            format!("{:?}", parse("{0} + {1}", &args).unwrap()),
            r#"Comp(Text("left"), Text("right"), Attr { pad: Padded, brk: Breakable })"#
        );
    }

    #[test]
    fn index_reuse() {
        let args = [text("x")];
        assert_eq!(
            format!("{:?}", parse("{0} & {0}", &args).unwrap()),
            r#"Comp(Text("x"), Text("x"), Attr { pad: Unpadded, brk: Breakable })"#
        );
    }

    #[test]
    fn whitespace_is_insignificant() {
        assert_eq!(
            parsed("\"a\"\n\t+ \"b\""),
            r#"Comp(Text("a"), Text("b"), Attr { pad: Padded, brk: Breakable })"#
        );
    }

    #[test]
    fn unterminated_string_is_rejected() {
        parse_err(r#""unterminated"#);
    }

    #[test]
    fn empty_input_is_rejected() {
        parse_err("");
    }

    #[test]
    fn trailing_operator_is_rejected() {
        parse_err(r#""a" +"#);
    }
}
