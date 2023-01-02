# External module dependencies
from dataclasses import dataclass

###############################################################################
# Typeset Layout AST
###############################################################################
@dataclass
class Layout: ...

@dataclass
class _Null(Layout): ...

@dataclass
class _Text(Layout):
    data : str

@dataclass
class _Fix(Layout):
    layout : Layout

@dataclass
class _Grp(Layout):
    layout : Layout

@dataclass
class _Seq(Layout):
    layout : Layout

@dataclass
class _Nest(Layout):
    layout : Layout

@dataclass
class _Pack(Layout):
    layout : Layout

@dataclass
class _Line(Layout):
    left : Layout
    right : Layout

@dataclass
class _Comp(Layout):
    left : Layout
    right : Layout
    pad : bool
    fix : bool

def null() -> Layout:
    return _Null()

def text(data : str) -> Layout:
    return _Text(data)

def fix(layout : Layout) -> Layout:
    return _Fix(layout)

def grp(layout : Layout) -> Layout:
    return _Grp(layout)

def seq(layout : Layout) -> Layout:
    return _Seq(layout)

def nest(layout : Layout) -> Layout:
    return _Nest(layout)

def pack(layout : Layout) -> Layout:
    return _Pack(layout)

def line(
    left : Layout,
    right : Layout
    ) -> Layout:
    return _Line(left, right)

def comp(
    left : Layout,
    right : Layout,
    pad : bool,
    fix : bool
    ) -> Layout:
    return _Comp(left, right, pad, fix)