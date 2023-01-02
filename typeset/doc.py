# External module dependencies
from dataclasses import dataclass, replace
from typing import Tuple, List, Dict

###############################################################################
# Typeset document AST
###############################################################################

@dataclass
class _EOD: pass

@dataclass
class _Empty:
    doc : 'Document'

@dataclass
class _Break:
    obj : 'DocumentObject'
    doc : 'Document'

@dataclass
class _Line:
    obj : 'DocumentObject'

Document = _EOD | _Empty | _Break | _Line

@dataclass
class _Text:
    data : str

@dataclass
class _Fix:
    fix : 'DocumentObjectFix'

@dataclass
class _Grp:
    obj : 'DocumentObject'

@dataclass
class _Seq:
    obj : 'DocumentObject'

@dataclass
class _Nest:
    obj : 'DocumentObject'

@dataclass
class _Pack:
    index : int
    obj : 'DocumentObject'

@dataclass
class _Comp:
    left : 'DocumentObject'
    right : 'DocumentObject'
    pad : bool

DocumentObject = _Text | _Fix | _Grp | _Seq | _Nest | _Pack | _Comp

@dataclass
class _FixText:
    data : str

@dataclass
class _FixComp:
    left : 'DocumentObjectFix'
    right : 'DocumentObjectFix'
    pad : bool

DocumentObjectFix = _FixText | _FixComp

###############################################################################
# Render function
###############################################################################
@dataclass
class _State:
    head : bool
    line : bool
    lvl : int
    pos : int
    marks : Dict[int, int]

def _init():
    return _State(
        head = True,
        line = False,
        lvl = 0,
        pos = 0,
        marks = {}
    )

def _inc_pos(state : _State, value : int) -> _State:
    return replace(state, pos = state.pos + value)

def _indent(tab : int, state : _State) -> _State:
    if tab <= 0: state
    lvl = state.lvl + (tab - (state.lvl % tab))
    return replace(state, lvl = lvl)

def _newline(state : _State) -> _State:
    return replace(state,
        head = True,
        pos = 0
    )

def _reset(state : _State) -> _State:
    return replace(state,
        head = True,
        line = False,
        pos = 0
    )

def _offset(state : _State) -> int:
    if not state.head: return 0
    return max(0, (state.lvl - state.pos))

def render(
    doc : Document,
    tab : int,
    width : int
    ) -> str:
    def _measure(
        obj : DocumentObject,
        state : _State
        ) -> int:
        ...
    def _next_comp(
        obj : DocumentObject,
        state : _State
        ) -> int:
        ...
    def _will_fit(
        obj : DocumentObject,
        state : _State
        ) -> bool:
        return _measure(obj, state) <= width
    def _should_break(
        obj : DocumentObject,
        state : _State
        ) -> bool:
        if state.line: return True
        return width < _next_comp(obj, state)
    def _visit_fix(
        fix : DocumentObjectFix,
        state : _State
        ) -> Tuple[_State, str]:
        while True:
            result : str = ''
            stack : List[Tuple[int, DocumentObjectFix]] = [(0, fix)]
            while len(stack) != 0:
                _pad, _fix = stack.pop(-1)
                match _fix:
                    case _FixText(data):
                        state = _inc_pos(state, len(data) + _pad)
                        result += (' ' * _pad) + data
                        continue
                    case _FixComp(left, right, pad):
                        fix = left
                        stack.append(right)
                        continue
            return state, result
    def _visit_obj(
        obj : DocumentObject,
        state : _State
        ) -> Tuple[_State, str]:
        ...
    def _visit_doc(
        doc : Document,
        state : _State
        ) -> Tuple[_State, str]:
        ...
    _, result = _visit_doc(doc, _init())
    return result