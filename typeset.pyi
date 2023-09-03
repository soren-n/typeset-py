class Layout:
    """
    A class representing a Layout.
    Can only be instanced via the packaged constructors.
    """

class Document:
    """
    A class representing a Document.
    Can only be instanced via the compile constructor.
    """

def null() -> Layout:
    """
    Construct a null layout.

    Returns:
        A null layout.
    """

def text(data: str) -> Layout:
    """
    Construct a text layout.

    Args:
        data: string to be wrapped and treated as an atomic layout unit.

    Returns:
        A text layout.
    """

def fix(layout: Layout) -> Layout:
    """
    Construct a fixed layout.

    Args:
        layout: a layout to be wrapped.

    Returns:
        A fixed layout.
    """

def grp(layout: Layout) -> Layout:
    """
    Construct a grouped layout.

    Args:
        layout: a layout to be wrapped.

    Returns:
        A grouped layout
    """

def seq(layout: Layout) -> Layout:
    """
    Construct a sequenced layout.

    Args:
        layout: a layout to be wrapped.

    Returns:
        A sequenced layout.
    """

def nest(layout: Layout) -> Layout:
    """
    Construct a nested layout.

    Args:
        layout: a layout to be wrapped.

    Returns:
        A nested layout.
    """

def pack(layout: Layout) -> Layout:
    """
    Construct a packed layout.

    Args:
        layout: a layout to be wrapped.

    Returns:
        A packed layout.
    """

def line(left: Layout, right: Layout) -> Layout:
    """
    Construct a forced line break layout composition.

    Args:
        left: a layout to be composed on the left hand side.
        right: a layout to be composed on the right hand side.

    Returns:
        A forced line break composed layout.
    """

def comp(left: Layout, right: Layout, padded: bool, fixed: bool) -> Layout:
    """
    Construct a potentially padded and fixed layout composition.

    Args:
        left: a layout to be composed on the left hand side..
        right: a layout to be composed on the right hand side..
        padded: padding state.
        fixed: fixed state.

    Returns:
        A potentially padded and fixed composed layout.
    """

def compile(layout: Layout) -> Document:
    """
    Compile a layout to construct a document.

    Args:
        layout: a layout to be compiled.

    Returns:
        A compiled document.
    """

def render(document: Document, indent: int, width: int) -> str:
    """
    Render a document to a string with a given indent and line width.

    Args:
        document: a document to be rendered.
        indent: the indent width to be rendered with.
        width: the line width to be rendered with.

    Returns:
        A rendered document as a string.
    """

def parse(input: str, *args: Layout) -> Layout:
    """
    Parse a typeset DSL script to construct a layout.

    Args:
        input: a typeset DSL script to be parsed.
        args: the arguments to be inserted in case of layout parameters.

    Returns:
        A layout representing the given DSL script.
    """