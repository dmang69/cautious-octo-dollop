#!/usr/bin/env python3
"""Q-RAM memory model with GF(4) ECC verification."""

from __future__ import annotations

from dataclasses import dataclass
from typing import List

BASE = 4
GF4_ADD = [0, 1, 2, 3]
GF4_MUL = [0, 0, 0, 0,
           0, 1, 2, 3,
           0, 2, 3, 1,
           0, 3, 1, 2]


def gf4_add(a: int, b: int) -> int:
    return a ^ b


def gf4_mul(a: int, b: int) -> int:
    index = a * BASE + b
    return GF4_MUL[index]


@dataclass
class GF4ECCBlock:
    data: List[int]
    parity: List[int]

    @classmethod
    def encode(cls, data: List[int]) -> "GF4ECCBlock":
        if len(data) != 3:
            raise ValueError("GF4ECC only supports 3-symbol payload blocks")
        p0 = gf4_add(gf4_add(data[0], data[1]), data[2])
        p1 = gf4_add(gf4_add(data[0], gf4_mul(2, data[1])), gf4_mul(3, data[2]))
        return cls(data=data, parity=[p0, p1])

    def codeword(self) -> List[int]:
        return self.data + self.parity

    @classmethod
    def decode(cls, codeword: List[int]) -> List[int]:
        if len(codeword) != 5:
            raise ValueError("Expected 5-symbol codeword")

        def encode_symbols(data_symbols: List[int]) -> List[int]:
            p0 = gf4_add(gf4_add(data_symbols[0], data_symbols[1]), data_symbols[2])
            p1 = gf4_add(gf4_add(data_symbols[0], gf4_mul(2, data_symbols[1])), gf4_mul(3, data_symbols[2]))
            return data_symbols + [p0, p1]

        if codeword == encode_symbols(codeword[:3]):
            return codeword[:3]

        for pos in range(len(codeword)):
            for err in range(1, BASE):
                candidate = codeword.copy()
                candidate[pos] = gf4_add(candidate[pos], err)
                if candidate == encode_symbols(candidate[:3]):
                    return candidate[:3]

        raise ValueError("Unable to correct codeword")


class QRAMBank:
    def __init__(self) -> None:
        self.rows = [[0] * 16 for _ in range(16)]

    def write(self, row: int, col: int, value: int) -> None:
        if not (0 <= row < 16 and 0 <= col < 16):
            raise IndexError("QRAM address out of range")
        self.rows[row][col] = value % BASE

    def read(self, row: int, col: int) -> int:
        if not (0 <= row < 16 and 0 <= col < 16):
            raise IndexError("QRAM address out of range")
        return self.rows[row][col]


def run_qram_ecc_verification() -> None:
    print("Q-RAM + GF(4) ECC verification: 960 error tests")
    bank = QRAMBank()
    tests = 0

    for row in range(16):
        for base_value in range(12):
            data_symbols = [row % BASE, base_value % BASE, (row + base_value) % BASE]
            block = GF4ECCBlock.encode(data_symbols)
            codeword = block.codeword()
            for error_slot in range(5):
                corrupted = codeword.copy()
                corrupted[error_slot] = (corrupted[error_slot] + 1) % BASE
                decoded = GF4ECCBlock.decode(corrupted)
                if decoded != data_symbols:
                    raise AssertionError(
                        f"ECC failed row={row} base={base_value} slot={error_slot}: "
                        f"{decoded} != {data_symbols}"
                    )
                tests += 1

    if tests != 960:
        raise AssertionError(f"Expected 960 ECC tests, got {tests}")

    print("Q-RAM verification complete: 960 GF(4) ECC tests passed.")


if __name__ == "__main__":
    run_qram_ecc_verification()
