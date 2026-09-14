//! The run-time front end of the layout DSL, with fragment substitution.
//!
//! The grammar lives upstream: `typeset::dsl::grammar` is the token parser
//! that `Layout`'s `FromStr` and the `layout!` macro share, and this module
//! is its third front end. It tokenizes the source the way upstream does,
//! with one addition: `{i}` is a variable naming the i-th fragment. So the
//! DSL Python sees is exactly the DSL upstream documents and prints.

use typeset::dsl::grammar::{Binary, Build, Token, Unary, parse_text, parse_tokens};
use typeset::{Break, Layout, Pad, comp, fix, grp, line, nest, null, pack, seq, text};

/// Parse a layout DSL script, substituting `{i}` with `fragments[i]`.
///
/// Errors carry the byte offset of the offending token.
pub fn parse(input: &str, fragments: &[Layout]) -> Result<Layout, String> {
    let tokens = tokenize(input, fragments.len())?;
    parse_tokens(tokens, input.len(), &mut Fragments(fragments)).map_err(|error| error.to_string())
}

/// The builder: the upstream constructors, and a variable is a fragment.
struct Fragments<'a>(&'a [Layout]);

impl Build for Fragments<'_> {
    type Var = usize;
    type Out = Layout;

    fn null(&mut self) -> Layout {
        null()
    }

    fn text(&mut self, data: String) -> Layout {
        text(data)
    }

    /// The tokenizer has already checked the index against the fragments.
    fn var(&mut self, index: usize) -> Layout {
        self.0[index].clone()
    }

    fn unary(&mut self, op: Unary, layout: Layout) -> Layout {
        match op {
            Unary::Fix => fix(layout),
            Unary::Grp => grp(layout),
            Unary::Seq => seq(layout),
            Unary::Nest => nest(layout),
            Unary::Pack => pack(layout),
        }
    }

    fn line(&mut self, left: Layout, right: Layout) -> Layout {
        line(left, right)
    }

    fn comp(&mut self, left: Layout, right: Layout, pad: Pad, brk: Break) -> Layout {
        comp(left, right, pad, brk)
    }
}

fn at(at: usize, message: impl std::fmt::Display) -> String {
    format!("{message} at byte {at}")
}

/// Splits `src` into the upstream tokens, each with its byte offset, and
/// checks every `{i}` against `fragment_count` so that substitution cannot
/// fail later.
fn tokenize(src: &str, fragment_count: usize) -> Result<Vec<(usize, Token<usize>)>, String> {
    let bytes = src.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        let token = match bytes[i] {
            b' ' | b'\t' | b'\n' | b'\r' => {
                i += 1;
                continue;
            }
            b'(' => {
                i += 1;
                Token::Open
            }
            b')' => {
                i += 1;
                Token::Close
            }
            b'"' => {
                let end =
                    literal_end(bytes, start).ok_or_else(|| at(start, "unterminated string"))?;
                i = end;
                let data = parse_text(&src[start..end])
                    .map_err(|error| at(start + error.at, error.message))?;
                Token::Text(data)
            }
            b'{' => {
                let close = src[start..]
                    .find('}')
                    .ok_or_else(|| at(start, "unterminated fragment index"))?;
                i = start + close + 1;
                Token::Var(
                    fragment_index(&src[start + 1..start + close], fragment_count)
                        .map_err(|message| at(start, message))?,
                )
            }
            b'a'..=b'z' => {
                while i < bytes.len() && bytes[i].is_ascii_lowercase() {
                    i += 1;
                }
                match &src[start..i] {
                    "null" => Token::Null,
                    word => Token::Unary(
                        Unary::from_keyword(word).ok_or_else(|| at(start, "unknown keyword"))?,
                    ),
                }
            }
            _ => {
                // Longest match first, so `!&` and `@@` are not split.
                let two = src.get(i..i + 2).and_then(Binary::from_symbol);
                let one = src.get(i..i + 1).and_then(Binary::from_symbol);
                let (len, op) = match (two, one) {
                    (Some(op), _) => (2, op),
                    (None, Some(op)) => (1, op),
                    (None, None) => return Err(at(start, "unexpected character")),
                };
                i += len;
                Token::Binary(op)
            }
        };
        tokens.push((start, token));
    }
    Ok(tokens)
}

/// The offset just past the closing quote of the string literal opening at
/// `start`, or `None` when the input ends first. Escapes are skipped as
/// pairs; validating them is `parse_text`'s job.
fn literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => return Some(i + 1),
            b'\\' => i += 2,
            _ => i += 1,
        }
    }
    None
}

/// Reads the digits between `{` and `}`: a decimal index without leading
/// zeros that names one of the `fragment_count` fragments.
fn fragment_index(digits: &str, fragment_count: usize) -> Result<usize, String> {
    let well_formed = !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'));
    if !well_formed {
        return Err("expected a fragment index".to_string());
    }
    let index: usize = digits
        .parse()
        .map_err(|_| format!("fragment index {digits} is out of range"))?;
    if index >= fragment_count {
        return Err(format!(
            "fragment index {index} is out of range; {fragment_count} fragment(s) given"
        ));
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use typeset::{Layout, text};

    /// The DSL the parsed layout prints as: upstream's `Display` is the
    /// normal form, so this pins the tree shape.
    fn parsed(input: &str) -> String {
        parse(input, &[]).unwrap().to_string()
    }

    fn parse_err(input: &str) -> String {
        parse(input, &[]).unwrap_err()
    }

    fn rendered(layout: Layout, width: usize) -> String {
        layout.compile().render(2, width)
    }

    #[test]
    fn null_literal() {
        assert_eq!(parsed("null"), r#""""#);
    }

    #[test]
    fn text_literal() {
        assert_eq!(parsed(r#""hello""#), r#""hello""#);
    }

    #[test]
    fn empty_text_literal() {
        assert_eq!(parsed(r#""""#), r#""""#);
    }

    #[test]
    fn escape_sequences() {
        let layout = parse(r#""a\nb\rc\td\\e\0f\"g\'h""#, &[]).unwrap();
        assert_eq!(rendered(layout, 80), "a\nb\rc\td\\e\0f\"g'h");
    }

    #[test]
    fn invalid_escape_is_rejected() {
        assert_eq!(parse_err(r#""\q""#), "unknown escape at byte 0");
    }

    #[test]
    fn unicode_text() {
        assert_eq!(parsed(r#""héllo" & "→""#), r#""héllo" & "→""#);
    }

    #[test]
    fn single_line_composition() {
        assert_eq!(parsed(r#""a" @ "b""#), r#""a" @ "b""#);
    }

    #[test]
    fn double_line_composition() {
        assert_eq!(parsed(r#""a" @@ "b""#), r#""a" @ "" @ "b""#);
    }

    #[test]
    fn compositions() {
        assert_eq!(parsed(r#""a" & "b""#), r#""a" & "b""#);
        assert_eq!(parsed(r#""a" + "b""#), r#""a" + "b""#);
        assert_eq!(parsed(r#""a" !& "b""#), r#""a" !& "b""#);
        assert_eq!(parsed(r#""a" !+ "b""#), r#""a" !+ "b""#);
    }

    #[test]
    fn binary_operators_are_right_associative() {
        assert_eq!(parsed(r#""a" + "b" + "c""#), r#""a" + "b" + "c""#);
        assert_eq!(parsed(r#""a" + ("b" + "c")"#), r#""a" + "b" + "c""#);
    }

    #[test]
    fn parentheses_override_associativity() {
        assert_eq!(parsed(r#"("a" + "b") + "c""#), r#"("a" + "b") + "c""#);
    }

    #[test]
    fn unary_operators() {
        assert_eq!(parsed(r#"fix "a""#), r#"fix "a""#);
        assert_eq!(parsed(r#"grp "a""#), r#"grp "a""#);
        assert_eq!(parsed(r#"seq "a""#), r#"seq "a""#);
        assert_eq!(parsed(r#"nest "a""#), r#"nest "a""#);
        assert_eq!(parsed(r#"pack "a""#), r#"pack "a""#);
    }

    #[test]
    fn stacked_unary_operators_need_parentheses() {
        // Upstream's grammar: a unary operator takes one primary.
        assert_eq!(parsed(r#"fix (grp "a")"#), r#"fix (grp "a")"#);
        assert_eq!(
            parse_err(r#"fix grp "a""#),
            "expected a primary expression at byte 4"
        );
    }

    #[test]
    fn unary_operator_over_parenthesized_expression() {
        assert_eq!(parsed(r#"fix ("a" + "b")"#), r#"fix ("a" + "b")"#);
    }

    #[test]
    fn unary_binds_tighter_than_binary() {
        assert_eq!(parsed(r#"fix "a" + "b""#), r#"fix "a" + "b""#);
        assert_eq!(rendered(parse(r#"fix "a" + "b""#, &[]).unwrap(), 1), "a\nb");
    }

    #[test]
    fn index_substitution() {
        let args = [text("left"), text("right")];
        assert_eq!(
            parse("{0} + {1}", &args).unwrap().to_string(),
            r#""left" + "right""#
        );
    }

    #[test]
    fn index_reuse() {
        let args = [text("x")];
        assert_eq!(
            parse("{0} & {0}", &args).unwrap().to_string(),
            r#""x" & "x""#
        );
    }

    #[test]
    fn fragments_are_spliced_as_subtrees() {
        let args = [parse(r#""a" + "b""#, &[]).unwrap()];
        assert_eq!(
            parse("nest {0} @ {0}", &args).unwrap().to_string(),
            r#"nest ("a" + "b") @ "a" + "b""#
        );
    }

    #[test]
    fn out_of_range_index_is_an_error() {
        let args = [text("x")];
        assert_eq!(
            parse("{1}", &args).unwrap_err(),
            "fragment index 1 is out of range; 1 fragment(s) given at byte 0"
        );
    }

    #[test]
    fn index_without_fragments_is_an_error() {
        assert_eq!(
            parse_err("{0}"),
            "fragment index 0 is out of range; 0 fragment(s) given at byte 0"
        );
    }

    #[test]
    fn index_beyond_usize_is_an_error_not_a_panic() {
        assert_eq!(
            parse_err("{99999999999999999999999}"),
            "fragment index 99999999999999999999999 is out of range at byte 0"
        );
    }

    #[test]
    fn malformed_indices_are_rejected() {
        assert_eq!(parse_err("{}"), "expected a fragment index at byte 0");
        assert_eq!(parse_err("{01}"), "expected a fragment index at byte 0");
        assert_eq!(parse_err("{x}"), "expected a fragment index at byte 0");
        assert_eq!(parse_err("{0"), "unterminated fragment index at byte 0");
    }

    #[test]
    fn whitespace_is_insignificant() {
        assert_eq!(parsed("\"a\"\n\t+ \"b\""), r#""a" + "b""#);
    }

    #[test]
    fn unterminated_string_is_rejected() {
        assert_eq!(
            parse_err(r#""unterminated"#),
            "unterminated string at byte 0"
        );
        assert_eq!(parse_err(r#""dangling\"#), "unterminated string at byte 0");
    }

    #[test]
    fn empty_input_is_rejected() {
        assert_eq!(parse_err(""), "expected a primary expression at byte 0");
    }

    #[test]
    fn trailing_operator_is_rejected() {
        assert_eq!(
            parse_err(r#""a" +"#),
            "expected a primary expression at byte 5"
        );
    }

    #[test]
    fn unknown_tokens_are_rejected() {
        assert_eq!(parse_err("foo"), "unknown keyword at byte 0");
        assert_eq!(parse_err(r#""a" ! "b""#), "unexpected character at byte 4");
        assert_eq!(parse_err(r#""a" "b""#), "expected an operator at byte 4");
    }
}
