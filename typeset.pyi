__version__: str

class Layout:
    """
    An unsolved layout tree.

    Instances are created via the module's constructor functions and
    composed with those functions or with the operators below.
    """

    def __add__(self, other: Layout) -> Layout:
        """Padded composition, equivalent to pad(self, other)."""

    def __and__(self, other: Layout) -> Layout:
        """Unpadded composition, equivalent to unpad(self, other)."""

    def __matmul__(self, other: Layout) -> Layout:
        """Forced linebreak composition, equivalent to line(self, other)."""

class Document:
    """
    A compiled, render-ready document.

    Instances are created via the compile function.
    """

def null() -> Layout:
    """
    Construct the empty layout.

    Null layouts are eliminated by the compiler and render to nothing.

    Returns:
        The empty layout.
    """

def text(data: str) -> Layout:
    """
    Construct a text layout: an atomic word literal.

    Args:
        data: string to be treated as an atomic layout unit.

    Returns:
        A text layout.
    """

def fix(layout: Layout) -> Layout:
    """
    Construct a fixed layout: its compositions are never broken.

    Args:
        layout: the layout to fix.

    Returns:
        A fixed layout.
    """

def grp(layout: Layout) -> Layout:
    """
    Construct a grouped layout: it does not break while compositions to
    its left can still be broken.

    Args:
        layout: the layout to group.

    Returns:
        A grouped layout.
    """

def seq(layout: Layout) -> Layout:
    """
    Construct a sequenced layout: if one of its compositions breaks,
    they all break.

    Args:
        layout: the layout to sequence.

    Returns:
        A sequenced layout.
    """

def nest(layout: Layout) -> Layout:
    """
    Construct a nested layout: one extra level of indentation for the
    literals it ranges over.

    Args:
        layout: the layout to nest.

    Returns:
        A nested layout.
    """

def pack(layout: Layout) -> Layout:
    """
    Construct a packed layout: indentation aligned to the buffer index
    of its first literal.

    Args:
        layout: the layout to pack.

    Returns:
        A packed layout.
    """

def line(left: Layout, right: Layout) -> Layout:
    """
    Compose two layouts with a forced linebreak.

    Args:
        left: the layout left of the break.
        right: the layout right of the break.

    Returns:
        The composed layout.
    """

def pad(left: Layout, right: Layout) -> Layout:
    """
    Compose two layouts separated by a space.

    Args:
        left: the left-hand layout.
        right: the right-hand layout.

    Returns:
        The composed layout.
    """

def unpad(left: Layout, right: Layout) -> Layout:
    """
    Compose two layouts with no separation.

    Args:
        left: the left-hand layout.
        right: the right-hand layout.

    Returns:
        The composed layout.
    """

def fix_pad(left: Layout, right: Layout) -> Layout:
    """
    Compose two layouts separated by a space, fixed at the seam: the
    adjacent literals never break apart.

    Args:
        left: the left-hand layout.
        right: the right-hand layout.

    Returns:
        The composed layout.
    """

def fix_unpad(left: Layout, right: Layout) -> Layout:
    """
    Compose two layouts with no separation, fixed at the seam: the
    adjacent literals never break apart.

    Args:
        left: the left-hand layout.
        right: the right-hand layout.

    Returns:
        The composed layout.
    """

def space() -> Layout:
    """
    A single space; equivalent to text(" ").

    Returns:
        A space layout.
    """

def comma() -> Layout:
    """
    A comma; equivalent to text(",").

    Returns:
        A comma layout.
    """

def semicolon() -> Layout:
    """
    A semicolon; equivalent to text(";").

    Returns:
        A semicolon layout.
    """

def newline() -> Layout:
    """
    A linebreak with no content on either side.

    Returns:
        A linebreak layout.
    """

def blank_line() -> Layout:
    """
    Two consecutive linebreaks, producing one blank line.

    Returns:
        A blank-line layout.
    """

def join_with(layouts: list[Layout], separator: Layout) -> Layout:
    """
    Join layouts with a separator between each pair, using unpadded
    compositions (the separator supplies its own spacing).

    Args:
        layouts: the layouts to join; an empty list yields null().
        separator: the separator layout.

    Returns:
        The joined layout.
    """

def join_with_spaces(layouts: list[Layout]) -> Layout:
    """
    Join layouts with single spaces.

    Args:
        layouts: the layouts to join; an empty list yields null().

    Returns:
        The joined layout.
    """

def join_with_commas(layouts: list[Layout]) -> Layout:
    """
    Join layouts with ", " separators.

    Args:
        layouts: the layouts to join; an empty list yields null().

    Returns:
        The joined layout.
    """

def join_with_lines(layouts: list[Layout]) -> Layout:
    """
    Join layouts with forced linebreaks, one element per line.

    Args:
        layouts: the layouts to join; an empty list yields null().

    Returns:
        The joined layout.
    """

def parens(layout: Layout) -> Layout:
    """
    Wrap a layout in parentheses with no introduced spacing.

    Args:
        layout: the layout to wrap.

    Returns:
        The wrapped layout.
    """

def brackets(layout: Layout) -> Layout:
    """
    Wrap a layout in square brackets with no introduced spacing.

    Args:
        layout: the layout to wrap.

    Returns:
        The wrapped layout.
    """

def braces(layout: Layout) -> Layout:
    """
    Wrap a layout in curly braces with no introduced spacing.

    Args:
        layout: the layout to wrap.

    Returns:
        The wrapped layout.
    """

def compile(layout: Layout) -> Document:
    """
    Compile a layout into a document.

    Args:
        layout: the layout to compile.

    Returns:
        The compiled document.
    """

def render(document: Document, tab: int, width: int) -> str:
    """
    Render a document to a string.

    Args:
        document: the document to render.
        tab: the number of spaces per indentation level.
        width: the target line width for breaking decisions.

    Returns:
        The rendered output.
    """

def format_layout(layout: Layout, tab: int, width: int) -> str:
    """
    Compile and render a layout in one step.

    To format the same layout repeatedly at different widths, prefer
    compile once and render per call.

    Args:
        layout: the layout to format.
        tab: the number of spaces per indentation level.
        width: the target line width for breaking decisions.

    Returns:
        The rendered output.
    """

def parse(input: str, *fragments: Layout) -> Layout:
    """
    Parse a typeset DSL script into a layout.

    Args:
        input: the DSL script to parse.
        fragments: layouts substituted for {i} placeholders by index.

    Returns:
        The parsed layout.

    Raises:
        ValueError: if the script does not parse or a fragment index is
            out of range.
    """
