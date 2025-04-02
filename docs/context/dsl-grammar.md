# DSL Grammar and Parsing

## Grammar Specification

The typeset DSL is defined using Pest grammar in `src/layout.pest`. Here's the complete grammar:

### Lexical Elements

```pest
WHITESPACE = _{ " " | "\t" | NEWLINE }

digit = { '0'..'9' }
non_zero = { '1'..'9' }
index = @{ "0" | non_zero ~ digit* }
```

### String Literals

```pest
raw_string = { (!("\\" | "\"") ~ ANY)+ }
predefined = { "n" | "r" | "t" | "\\" | "0" | "\"" | "'" }
escaped_string = { "\\" ~ predefined }
text = { (raw_string | escaped_string)* }
string = _{ "\"" ~ text ~ "\"" }
```

**Escape Sequences:**
- `\n` → newline
- `\r` → carriage return
- `\t` → tab
- `\\` → backslash
- `\0` → null character
- `\"` → quote
- `\'` → apostrophe

### Operators

#### Unary Operators (Prefix)
```pest
fix_op = { "fix" }      # Fixed layout (no breaking)
grp_op = { "grp" }      # Group (prefer not to break)
seq_op = { "seq" }      # Sequence (all break together)
nest_op = { "nest" }    # Nested indentation
pack_op = { "pack" }    # Pack to first element position
```

#### Binary Operators (Infix)
```pest
single_line_op = { "@" }        # Force single line break
double_line_op = { "@@" }       # Force double line break
unpad_comp_op = { "&" }         # Unpadded composition
pad_comp_op = { "+" }           # Padded composition
fix_unpad_comp_op = { "!&" }    # Fixed unpadded composition
fix_pad_comp_op = { "!+" }      # Fixed padded composition
```

### Operator Precedence

The Pratt parser defines precedence (highest to lowest):

1. **Prefix operators**: `fix`, `grp`, `seq`, `nest`, `pack`
2. **Binary operators**: `@`, `@@`, `&`, `+`, `!&`, `!+` (right-associative)

### Expression Structure

```pest
primary = _{ null | variable | string | "(" ~ expr ~ ")" }
atom = _{ unary_op? ~ primary }
expr = { atom ~ (binary_op ~ atom)* }
layout = _{ SOI ~ expr ~ EOI }
```

### Variables and Substitution

```pest
variable = _{ "{" ~ index ~ "}" }
```

Variables like `{0}`, `{1}`, etc. are substituted with arguments passed to `parse()`.

## DSL Semantics

### Layout Constructors

#### Text Literals
```python
parse('"hello world"')  # Creates text layout
```

#### Null Layouts
```python
parse('null')  # Creates empty layout (eliminated during compilation)
```

#### Composition Operations

**Padded Composition (`+`)**:
```python
parse('"hello" + "world"')  # "hello world" (with space)
```

**Unpadded Composition (`&`)**:
```python
parse('"hello" & "world"')  # "helloworld" (no space)
```

**Fixed Compositions (`!+`, `!&`)**:
```python
parse('"hello" !+ "world"')  # Forces hello and world to stay together
```

#### Line Breaks

**Single Line Break (`@`)**:
```python
parse('"line1" @ "line2"')  # Forces line break between elements
```

**Double Line Break (`@@`)**:
```python
parse('"para1" @@ "para2"')  # Forces blank line between elements
```

#### Layout Modifiers

**Fixed Layout (`fix`)**:
```python
parse('fix ("a" + "b" + "c")')  # Treats entire expression as atomic
```

**Grouped Layout (`grp`)**:
```python
parse('"start" + grp ("a" + "b") + "end"')  # Prefers not to break group
```

**Sequence Layout (`seq`)**:
```python
parse('seq ("a" + "b" + "c")')  # All break together or none break
```

**Nested Layout (`nest`)**:
```python
parse('"func" + nest ("arg1" + "arg2")')  # Indents nested content
```

**Packed Layout (`pack`)**:
```python
parse('"func" + pack ("arg1" + "arg2")')  # Aligns to first element position
```

## Parsing Implementation

### Syntax Tree

The parser builds an intermediate `Syntax` enum:

```rust
enum Syntax {
    Null,
    Index(usize),                                    // Variable substitution
    Text(String),                                    // String literal
    Fix(Box<Syntax>),                               // Layout modifiers...
    Grp(Box<Syntax>),
    Seq(Box<Syntax>),
    Nest(Box<Syntax>),
    Pack(Box<Syntax>),
    SingleLine(Box<Syntax>, Box<Syntax>),           // Binary operations...
    DoubleLine(Box<Syntax>, Box<Syntax>),
    UnpadComp(Box<Syntax>, Box<Syntax>),
    PadComp(Box<Syntax>, Box<Syntax>),
    FixUnpadComp(Box<Syntax>, Box<Syntax>),
    FixPadComp(Box<Syntax>, Box<Syntax>),
}
```

### Two-Pass Processing

1. **Parse Phase**: Text → `Syntax` tree using Pratt parser
2. **Interpretation Phase**: `Syntax` tree → `Layout` with variable substitution

### Variable Substitution

```rust
box Syntax::Index(index) => {
    if index < args.len() {
        Ok(args[index].clone())
    } else {
        Err(format!("invalid index {:?}", index))
    }
}
```

Variables `{n}` are replaced with the nth argument passed to `parse()`.

## Advanced DSL Patterns

### Conditional Layouts
```python
# Using null for optional content
args = [text("optional") if condition else null()]
layout = parse('"start" + {0} + "end"', *args)
```

### Template Patterns
```python
# Function call template
def func_call(name, args):
    return parse('{0} + "(" + {1} + ")"', text(name), args)
```

### Recursive Structures
```python
# Pretty printing lists
def pretty_list(items):
    if not items:
        return parse('"[]"')

    sep = parse('", "')
    content = seq_composition(items, sep)  # Helper function
    return parse('"[" + nest {0} + "]"', content)
```

## Error Handling

### Parse Errors
- **Invalid syntax**: Malformed expressions
- **Invalid escapes**: Unknown escape sequences
- **Unmatched parentheses**: Missing closing parentheses

### Runtime Errors
- **Invalid index**: Variable index out of bounds
- **Type errors**: Wrong argument types to `parse()`

### Error Messages
The parser provides detailed error context:
```
error: expected atom, found ':'
  --> input:1:8
   |
1 | "hello" : "world"
   |         ^
```

## Grammar Extensions

### Adding New Operators

1. **Define in grammar**: Add to `layout.pest`
2. **Add to Syntax enum**: New variant in `parser.rs`
3. **Update Pratt parser**: Set precedence and associativity
4. **Implement semantics**: Add interpretation logic

### Adding New Constructors

1. **Add grammar rule**: New unary operator
2. **Extend Syntax enum**: New constructor variant
3. **Add parsing logic**: Handle in `map_prefix`
4. **Add interpretation**: Convert to native Layout
