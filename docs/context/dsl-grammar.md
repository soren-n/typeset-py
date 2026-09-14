# DSL grammar

The layout DSL is upstream's: `typeset::dsl` documents it, and
`typeset::Layout` parses (`FromStr`) and prints (`Display`) it. This
repository adds one form, `{i}`, and otherwise defers to upstream's token
parser (`src/parser.rs`). Every form below is covered by the unit tests in
`src/parser.rs`; if this document and the tests disagree, the tests win.

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
u @@ v    forced double linebreak (one blank line): `u @ null @ v`
u & v     unpadded composition
u !& v    unpadded composition, fixed at the seam
u + v     padded composition
u !+ v    padded composition, fixed at the seam
(u)       grouping
```

## Structure

- A unary constructor takes one primary (`null`, a literal, a fragment, or
  a parenthesized expression), so it binds tighter than binary operators:
  `fix "a" + "b"` is `(fix "a") + "b"`. Stacking needs parentheses:
  `fix (grp "a")`, not `fix grp "a"`.
- All binary operators share a single precedence level and associate to
  the right: `"a" + "b" + "c"` is `"a" + ("b" + "c")`.
- Whitespace (spaces, tabs, newlines) is insignificant between tokens.

## Text literals

Strings are double-quoted. Escape sequences: `\n` `\r` `\t` `\\` `\0`
`\"` `\'`. Any other escape is a parse error. The empty string `""` is a
valid literal, and is the same layout as `null`.

## Fragment indices

`{i}` takes a zero-based decimal index without leading zeros. An index
with no matching fragment argument, or one too large for the platform,
is a `ValueError` - never a crash. A fragment is spliced as a subtree, so
`nest {0}` wraps the whole fragment however it was composed.

## Errors

Every error names the byte offset of the offending token, e.g.
`expected a primary expression at byte 5`. The messages for everything
but `{i}` are upstream's.

## Printing

`repr(layout)` is the DSL, and `parse(repr(layout))` is the same layout.
The printer parenthesizes only where the grammar needs it: a binary left
operand that is itself binary, and a unary operand that is not a literal.
