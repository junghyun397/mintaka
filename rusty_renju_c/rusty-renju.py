from enum import Enum


class RustyRenjuLib:
    pass


class BoardDescribe:
    pass


class Pos:
    def __init__(self, index: int):
        self.__index = index

    @classmethod
    def from_index(cls, index: int) -> 'Pos':
        return cls(index)

    @classmethod
    def from_cartesian(cls, row: int, col: int) -> 'Pos':
        return cls(row * 15 + col)

    def index(self) -> int:
        return self.__index

    def row(self) -> int:
        return self.__index // 15

    def col(self) -> int:
        return self.__index % 15


class Color(Enum):
    BLACK = 0
    WHITE = 1

    def flip(self) -> 'Color':
        return Color.WHITE if self == Color.BLACK else Color.BLACK

    def __str__(self) -> str:
        return self.name.lower()


class RuleKind(Enum):
    RENJU = 0
    GOMOKU = 1
    FREESTYLE = 2

    def __str__(self) -> str:
        return self.name.lower()


class Board:
    def __init__(self, lib: RustyRenjuLib, rule: RuleKind, ptr: int):
        self.__lib = lib
        self.__ptr = ptr
        self.__describe = None
        self.rule = rule

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    @classmethod
    def empty(cls) -> 'Board':
        pass

    @classmethod
    def from_history(cls, history: list[Pos]) -> 'Board':
        pass

    def close(self):
        if self.__ptr:
            pass

    def set(self, color: Color, pos: Pos) -> 'Board':
        pass

    def unset(self, color: Color, pos: Pos) -> 'Board':
        pass
