#!/usr/bin/env python3
"""Q-ALU simulation and verification harness."""

from __future__ import annotations

from dataclasses import dataclass
from typing import List

BASE = 4

@dataclass(frozen=True)
class QWord:
    qits: List[int]

    @classmethod
    def from_int(cls, value: int, width_qits: int) -> "QWord":
        mask = BASE - 1
        qits = [(value >> (2 * i)) & mask for i in range(width_qits)]
        return cls(qits)

    def to_int(self) -> int:
        value = 0
        for i, q in enumerate(self.qits):
            value |= (q & (BASE - 1)) << (2 * i)
        return value

    def normalized(self, width_qits: int) -> "QWord":
        if len(self.qits) >= width_qits:
            return QWord(self.qits[:width_qits])
        return QWord(self.qits + [0] * (width_qits - len(self.qits)))

    def __repr__(self) -> str:
        return f"QWord({self.qits})"


class QALU:
    def __init__(self, width: int) -> None:
        self.width = width
        self.modulus = BASE ** width

    def ADD(self, a: QWord, b: QWord) -> QWord:
        return self._from_int((a.to_int() + b.to_int()) % self.modulus)

    def SUB(self, a: QWord, b: QWord) -> QWord:
        return self._from_int((a.to_int() - b.to_int()) % self.modulus)

    def MUL(self, a: QWord, b: QWord) -> QWord:
        return self._from_int((a.to_int() * b.to_int()) % self.modulus)

    def MIN(self, a: QWord, b: QWord) -> QWord:
        return self._elementwise_min(a, b)

    def MAX(self, a: QWord, b: QWord) -> QWord:
        return self._elementwise_max(a, b)

    def XOR(self, a: QWord, b: QWord) -> QWord:
        return self._elementwise_op(a, b, lambda x, y: x ^ y)

    def NEG(self, a: QWord) -> QWord:
        return self._from_int((-a.to_int()) % self.modulus)

    def SHL(self, a: QWord, bits: int) -> QWord:
        return self._from_int((a.to_int() << bits) % self.modulus)

    def SHR(self, a: QWord, bits: int) -> QWord:
        return self._from_int(a.to_int() >> bits)

    def EQ(self, a: QWord, b: QWord) -> QWord:
        return self._from_int(1 if a.to_int() == b.to_int() else 0)

    def NE(self, a: QWord, b: QWord) -> QWord:
        return self._from_int(1 if a.to_int() != b.to_int() else 0)

    def _elementwise_op(self, a: QWord, b: QWord, op) -> QWord:
        result = [op(x, y) & (BASE - 1) for x, y in zip(a.qits, b.qits)]
        return QWord(result)

    def _elementwise_min(self, a: QWord, b: QWord) -> QWord:
        return QWord([min(x, y) for x, y in zip(a.qits, b.qits)])

    def _elementwise_max(self, a: QWord, b: QWord) -> QWord:
        return QWord([max(x, y) for x, y in zip(a.qits, b.qits)])

    def _from_int(self, value: int) -> QWord:
        return QWord.from_int(value, self.width)


def run_verification() -> None:
    width = 8
    alu = QALU(width)

    a = QWord.from_int(0, width)
    b = QWord.from_int(1, width)

    print("Q-ALU verification: 65,536 addition cases")
    for x in range(256):
        for y in range(256):
            left = QWord.from_int(x, width)
            right = QWord.from_int(y, width)
            expected = (x + y) % (4 ** width)
            result = alu.ADD(left, right).to_int()
            if result != expected:
                raise AssertionError(f"ADD failed for {x},{y}: {result} != {expected}")

    print("Q-ALU operation spot checks")
    sample_pairs = [(7, 3), (12, 30), (0, 255), (255, 1), (42, 42)]
    for x, y in sample_pairs:
        left = QWord.from_int(x, width)
        right = QWord.from_int(y, width)
        assert alu.SUB(left, right).to_int() == (x - y) % alu.modulus
        assert alu.MUL(left, right).to_int() == (x * y) % alu.modulus
        assert alu.MIN(left, right).qits == [min(u, v) for u, v in zip(left.qits, right.qits)]
        assert alu.MAX(left, right).qits == [max(u, v) for u, v in zip(left.qits, right.qits)]
        assert alu.XOR(left, right).qits == [u ^ v for u, v in zip(left.qits, right.qits)]
        assert alu.EQ(left, right).to_int() == (1 if x == y else 0)
        assert alu.NE(left, right).to_int() == (1 if x != y else 0)

    print("Q-ALU verification complete: 65,536 cases passed.")


if __name__ == "__main__":
    run_verification()
