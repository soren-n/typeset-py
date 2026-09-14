import pytest

import typeset


def render(layout: typeset.Layout, tab: int = 2, width: int = 80) -> str:
    return layout.compile().render(tab, width)


def test_text_renders_verbatim() -> None:
    assert render(typeset.text("hello")) == "hello"


def test_null_is_eliminated() -> None:
    layout = typeset.parse('"foo" & null & "bar"')
    assert render(layout) == "foobar"


def test_parse_padded_composition() -> None:
    assert render(typeset.parse('"a" + "b"')) == "a b"


def test_parse_unpadded_composition() -> None:
    assert render(typeset.parse('"a" & "b"')) == "ab"


def test_parse_forced_linebreak() -> None:
    assert render(typeset.parse('"a" @ "b"')) == "a\nb"


def test_parse_double_linebreak() -> None:
    assert render(typeset.parse('"a" @@ "b"')) == "a\n\nb"


def test_parse_fragment_substitution() -> None:
    layout = typeset.parse(
        "{0} + {1}", typeset.text("left"), typeset.text("right")
    )
    assert render(layout) == "left right"


def test_padded_composition_breaks_at_width() -> None:
    layout = typeset.parse('"foo" + "bar"')
    assert render(layout, width=4) == "foo\nbar"


def test_nest_indents_after_break() -> None:
    layout = typeset.parse('"foo" + nest ("bar" + "baz")')
    assert render(layout, tab=2, width=7) == "foo bar\n  baz"


def test_fix_prevents_breaking() -> None:
    layout = typeset.parse('fix ("foo" + "bar")')
    assert render(layout, width=4) == "foo bar"


def test_parse_error_raises_value_error() -> None:
    with pytest.raises(ValueError, match="unterminated string at byte 0"):
        typeset.parse('"unterminated')


def test_missing_fragment_raises_value_error() -> None:
    with pytest.raises(ValueError, match="fragment index 0 is out of range"):
        typeset.parse("{0}")


def test_huge_fragment_index_raises_value_error() -> None:
    with pytest.raises(ValueError):
        typeset.parse("{99999999999999999999999}")


def test_non_layout_fragment_raises_type_error() -> None:
    with pytest.raises(TypeError):
        typeset.parse("{0}", "not a layout")  # type: ignore[arg-type]


def test_layout_repr_is_the_dsl() -> None:
    layout = typeset.parse('nest ("a" + "b") @ "c"')
    assert repr(layout) == 'nest ("a" + "b") @ "c"'
    assert render(typeset.parse(repr(layout))) == render(layout)


def test_document_repr_is_the_dsl() -> None:
    document = typeset.parse('"a" + "b"').compile()
    assert render(typeset.parse(repr(document))) == "a b"


def test_version_matches_package_metadata() -> None:
    from importlib.metadata import version

    assert typeset.__version__ == version("typeset-soren-n")


def test_add_operator_is_padded_composition() -> None:
    assert render(typeset.text("a") + typeset.text("b")) == "a b"


def test_and_operator_is_unpadded_composition() -> None:
    assert render(typeset.text("a") & typeset.text("b")) == "ab"


def test_matmul_operator_is_forced_linebreak() -> None:
    assert render(typeset.text("a") @ typeset.text("b")) == "a\nb"


def test_pad_composition() -> None:
    assert render(typeset.pad(typeset.text("a"), typeset.text("b"))) == "a b"


def test_unpad_composition() -> None:
    assert render(typeset.unpad(typeset.text("a"), typeset.text("b"))) == "ab"


def test_fix_unpad_never_breaks_at_the_seam() -> None:
    layout = typeset.fix_unpad(typeset.text("foo"), typeset.text("bar"))
    assert render(layout, width=2) == "foobar"


def test_fix_pad_never_breaks_at_the_seam() -> None:
    layout = typeset.fix_pad(typeset.text("foo"), typeset.text("bar"))
    assert render(layout, width=2) == "foo bar"


def test_blank_line_is_a_line_onto_null() -> None:
    a, b = typeset.text("a"), typeset.text("b")
    assert render(a @ typeset.null() @ b) == "a\n\nb"


def test_join_with_spaces() -> None:
    items = [typeset.text("a"), typeset.text("b")]
    assert render(typeset.join_with_spaces(items)) == "a b"
    assert render(typeset.join_with_spaces(items), width=1) == "a\nb"


def test_join_with_commas() -> None:
    items = [typeset.text("a"), typeset.text("b"), typeset.text("c")]
    assert render(typeset.join_with_commas(items)) == "a, b, c"
    assert render(typeset.join_with_commas(items), width=3) == "a,\nb,\nc"


def test_join_with_lines() -> None:
    items = [typeset.text("a;"), typeset.text("b;")]
    assert render(typeset.join_with_lines(items)) == "a;\nb;"


def test_join_of_empty_list_is_null() -> None:
    assert render(typeset.join_with_commas([])) == ""


def test_document_renders_at_several_widths() -> None:
    args = typeset.pack(typeset.seq(typeset.join_with_commas(
        [typeset.text("x"), typeset.text("y"), typeset.text("z")]
    )))
    call = typeset.text("f(") & typeset.fix_unpad(args, typeset.text(")"))
    document = call.compile()
    assert document.render(2, 80) == "f(x, y, z)"
    assert document.render(2, 6) == "f(x,\n  y,\n  z)"


def test_layouts_are_reusable() -> None:
    separator = typeset.text(",")
    first = typeset.fix_unpad(typeset.text("a"), separator) + typeset.text("b")
    second = typeset.fix_unpad(typeset.text("c"), separator) + typeset.text("d")
    assert render(first) == "a, b"
    assert render(second) == "c, d"


def test_keyword_arguments_match_stub_names() -> None:
    layout = typeset.pad(left=typeset.text("a"), right=typeset.text("b"))
    assert layout.compile().render(tab=2, width=80) == "a b"
    assert render(typeset.parse(input='"x"')) == "x"
    assert render(typeset.join_with_spaces(layouts=[typeset.text("y")])) == "y"


def test_removed_helpers_are_gone() -> None:
    for name in (
        "comp", "print", "compile", "render", "format_layout", "join_with",
        "space", "comma", "semicolon", "newline", "blank_line",
        "parens", "brackets", "braces",
    ):
        assert not hasattr(typeset, name)


def test_deep_layouts_compile_and_free_in_constant_stack() -> None:
    import functools
    import operator

    n = 100_000
    words = [typeset.text(str(i)) for i in range(n)]
    one_line = " ".join(str(i) for i in range(n))
    left_deep = functools.reduce(operator.add, words)
    right_deep = functools.reduce(lambda acc, word: word + acc, reversed(words))
    assert render(left_deep, width=len(one_line)) == one_line
    assert render(right_deep, width=len(one_line)) == one_line
    nested = words[0]
    for _ in range(n):
        nested = typeset.nest(nested)
    assert render(nested) == " " * (2 * n) + "0"
    assert repr(nested).count("nest (") == n - 1
    del left_deep, right_deep, nested
