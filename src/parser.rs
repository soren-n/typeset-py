use std::sync::LazyLock;

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest_derive::Parser;

use typeset::{Break, Layout, Pad, comp, fix, grp, line, nest, null, pack, seq, text};

#[derive(Parser)]
#[grammar = "layout.pest"]
pub struct LayoutParser;

static PRATT: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::double_line_op, Assoc::Right)
            | Op::infix(Rule::single_line_op, Assoc::Right)
            | Op::infix(Rule::fix_unpad_comp_op, Assoc::Right)
            | Op::infix(Rule::unpad_comp_op, Assoc::Right)
            | Op::infix(Rule::fix_pad_comp_op, Assoc::Right)
            | Op::infix(Rule::pad_comp_op, Assoc::Right))
        .op(Op::prefix(Rule::fix_op)
            | Op::prefix(Rule::grp_op)
            | Op::prefix(Rule::seq_op)
            | Op::prefix(Rule::nest_op)
            | Op::prefix(Rule::pack_op))
});

/// Parse a layout DSL script, substituting `{i}` with `fragments[i]`.
pub fn parse(input: &str, fragments: &[Box<Layout>]) -> Result<Box<Layout>, String> {
    let mut pairs = LayoutParser::parse(Rule::layout, input).map_err(|error| error.to_string())?;
    build(pairs.next().unwrap().into_inner(), fragments)
}

fn build(pairs: Pairs<Rule>, fragments: &[Box<Layout>]) -> Result<Box<Layout>, String> {
    PRATT
        .map_primary(|primary| match primary.as_rule() {
            Rule::null => Ok(null()),
            Rule::index => substitute(primary, fragments),
            Rule::text => Ok(text(unescape(primary))),
            Rule::expr => build(primary.into_inner(), fragments),
            rule => unreachable!("grammar produced unexpected primary: {rule:?}"),
        })
        .map_prefix(|op, layout| {
            let constructor = match op.as_rule() {
                Rule::fix_op => fix,
                Rule::grp_op => grp,
                Rule::seq_op => seq,
                Rule::nest_op => nest,
                Rule::pack_op => pack,
                rule => unreachable!("grammar produced unexpected prefix operator: {rule:?}"),
            };
            Ok(constructor(layout?))
        })
        .map_infix(|left, op, right| {
            let (left, right) = (left?, right?);
            Ok(match op.as_rule() {
                Rule::single_line_op => line(left, right),
                Rule::double_line_op => line(left, line(null(), right)),
                Rule::unpad_comp_op => comp(left, right, Pad::Unpadded, Break::Breakable),
                Rule::pad_comp_op => comp(left, right, Pad::Padded, Break::Breakable),
                Rule::fix_unpad_comp_op => comp(left, right, Pad::Unpadded, Break::Fixed),
                Rule::fix_pad_comp_op => comp(left, right, Pad::Padded, Break::Fixed),
                rule => unreachable!("grammar produced unexpected infix operator: {rule:?}"),
            })
        })
        .parse(pairs)
}

fn substitute(pair: Pair<Rule>, fragments: &[Box<Layout>]) -> Result<Box<Layout>, String> {
    let index: usize = pair
        .as_str()
        .parse()
        .map_err(|_| format!("fragment index {} is out of range", pair.as_str()))?;
    fragments.get(index).cloned().ok_or_else(|| {
        format!(
            "fragment index {index} is out of range; {} fragment(s) given",
            fragments.len()
        )
    })
}

fn unescape(pair: Pair<Rule>) -> String {
    let mut result = String::new();
    for part in pair.into_inner() {
        match part.as_rule() {
            Rule::raw_string => result.push_str(part.as_str()),
            Rule::escaped_string => result.push(match &part.as_str()[1..] {
                "n" => '\n',
                "r" => '\r',
                "t" => '\t',
                "\\" => '\\',
                "0" => '\0',
                "\"" => '"',
                "'" => '\'',
                other => unreachable!("grammar produced unexpected escape: \\{other}"),
            }),
            rule => unreachable!("grammar produced unexpected text part: {rule:?}"),
        }
    }
    result
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
    fn double_line_composition() {
        assert_eq!(
            parsed(r#""a" @@ "b""#),
            r#"Line(Text("a"), Line(Null, Text("b")))"#
        );
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
    fn stacked_unary_operators() {
        assert_eq!(parsed(r#"fix grp "a""#), r#"Fix(Grp(Text("a")))"#);
        assert_eq!(
            parsed(r#"nest seq grp "a""#),
            r#"Nest(Seq(Grp(Text("a"))))"#
        );
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
    fn out_of_range_index_is_an_error() {
        let args = [text("x")];
        let error = parse("{1}", &args).unwrap_err();
        assert_eq!(
            error,
            "fragment index 1 is out of range; 1 fragment(s) given"
        );
    }

    #[test]
    fn index_without_fragments_is_an_error() {
        assert_eq!(
            parse_err("{0}"),
            "fragment index 0 is out of range; 0 fragment(s) given"
        );
    }

    #[test]
    fn index_beyond_usize_is_an_error_not_a_panic() {
        assert_eq!(
            parse_err("{99999999999999999999999}"),
            "fragment index 99999999999999999999999 is out of range"
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
