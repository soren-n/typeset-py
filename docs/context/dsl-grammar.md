# DSL grammar

The layout DSL is parsed at runtime by `src/parser.rs` against the Pest
grammar in `src/layout.pest`. Every form below is covered by the parser
unit tests in `src/parser.rs`; if this document and the tests disagree,
the tests win.

## Forms

```text
{i}       fragment substitution: replaced by the i-th Layout argument to parse()
null      the empty layout (eliminated by the compiler)
"x"       text literal
fix u     fixed layout: compositions inside are never broken
grp u     grouped layout: does not break while compositions to its left can
seq u     sequenced layout: if one composition breaks, all break
nest u    nested layout: one extra indentation level
pack u    packed layout: indentation aligned to the first literal's column
u @ v     forced linebreak composition
u @@ v    forced double linebreak (one blank line)
u & v     unpadded composition
u !& v    unpadded composition, fixed at the seam
u + v     padded composition
u !+ v    padded composition, fixed at the seam
(u)       grouping
```

## Structure

- Unary constructors stack (`fix grp "a"`) and bind tighter than binary
  operators: `fix "a" + "b"` is `(fix "a") + "b"`.
- All binary operators share a single precedence level and associate to
  the right: `"a" + "b" + "c"` is `"a" + ("b" + "c")`.
- Whitespace (spaces, tabs, newlines) is insignificant between tokens.

## Text literals

Strings are double-quoted. Escape sequences: `\n` `\r` `\t` `\\` `\0`
`\"` `\'`. Any other escape is a parse error. The empty string `""` is a
valid literal.

## Fragment indices

`{i}` takes a zero-based decimal index without leading zeros. An index
with no matching fragment argument, or one too large for the platform,
is a `ValueError` - never a crash.

## Grammar gotcha

Pest ordered choice commits to the first matching alternative, so
alternatives in `binary_op` are ordered longest-first: `@@` before `@`,
`!&` before `&`. Keep that ordering when adding operators.
