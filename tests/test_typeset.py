import pytest

import typeset


def render(layout: typeset.Layout, tab: int = 2, width: int = 80) -> str:
    return typeset.render(typeset.compile(layout), tab, width)


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
    with pytest.raises(ValueError):
        typeset.parse('"unterminated')


def test_missing_fragment_raises_value_error() -> None:
    with pytest.raises(ValueError):
        typeset.parse("{0}")


def test_huge_fragment_index_raises_value_error() -> None:
    with pytest.raises(ValueError):
        typeset.parse("{99999999999999999999999}")


def test_non_layout_fragment_raises_type_error() -> None:
    with pytest.raises(TypeError):
        typeset.parse("{0}", "not a layout")  # type: ignore[arg-type]


def test_layout_repr() -> None:
    assert repr(typeset.text("x")) == 'Text("x")'


def test_document_repr() -> None:
    assert "x" in repr(typeset.compile(typeset.text("x")))


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


def test_space_comma_semicolon() -> None:
    assert render(typeset.space()) == " "
    assert render(typeset.comma()) == ","
    assert render(typeset.semicolon()) == ";"


def test_newline_and_blank_line() -> None:
    a, b = typeset.text("a"), typeset.text("b")
    assert render(a & typeset.newline() & b) == "a\nb"
    assert render(a & typeset.blank_line() & b) == "a\n\nb"


def test_join_with() -> None:
    items = [typeset.text("a"), typeset.text("b")]
    assert render(typeset.join_with(items, typeset.text("|"))) == "a|b"


def test_join_with_empty_list_is_null() -> None:
    assert render(typeset.join_with([], typeset.comma())) == ""


def test_join_with_spaces() -> None:
    items = [typeset.text("a"), typeset.text("b")]
    assert render(typeset.join_with_spaces(items)) == "a b"


def test_join_with_commas() -> None:
    items = [typeset.text("a"), typeset.text("b"), typeset.text("c")]
    assert render(typeset.join_with_commas(items)) == "a, b, c"


def test_join_with_lines() -> None:
    items = [typeset.text("a;"), typeset.text("b;")]
    assert render(typeset.join_with_lines(items)) == "a;\nb;"


def test_wrappers() -> None:
    inner = typeset.text("x")
    assert render(typeset.parens(inner)) == "(x)"
    assert render(typeset.brackets(inner)) == "[x]"
    assert render(typeset.braces(inner)) == "{x}"


def test_format_layout_matches_compile_render() -> None:
    layout = typeset.parse('"foo" + nest ("bar" + "baz")')
    assert typeset.format_layout(layout, 2, 7) == render(layout, tab=2, width=7)


def test_layouts_are_reusable() -> None:
    separator = typeset.comma() & typeset.space()
    first = typeset.join_with([typeset.text("a"), typeset.text("b")], separator)
    second = typeset.join_with([typeset.text("c"), typeset.text("d")], separator)
    assert render(first) == "a, b"
    assert render(second) == "c, d"


def test_keyword_arguments_match_stub_names() -> None:
    document = typeset.compile(typeset.pad(left=typeset.text("a"), right=typeset.text("b")))
    assert typeset.render(document=document, tab=2, width=80) == "a b"
    layout = typeset.parse(input='"x"')
    assert typeset.format_layout(layout=layout, tab=2, width=80) == "x"


def test_boolean_comp_and_print_are_gone() -> None:
    assert not hasattr(typeset, "comp")
    assert not hasattr(typeset, "print")
