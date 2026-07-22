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


def test_non_layout_fragment_raises_type_error() -> None:
    with pytest.raises(TypeError):
        typeset.parse("{0}", "not a layout")  # type: ignore[arg-type]


def test_layout_repr() -> None:
    assert repr(typeset.text("x")) == 'Text("x")'


def test_document_repr() -> None:
    assert "x" in repr(typeset.compile(typeset.text("x")))
