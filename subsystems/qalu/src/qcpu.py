#!/usr/bin/env python3
"""QCPU fetch-decode-execute integration simulator."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, List

BASE = 4

@dataclass
class QInstruction:
    opcode: int
    a: int
    b: int
    c: int

    @classmethod
    def encode(cls, opcode: int, a: int, b: int, c: int) -> "QInstruction":
        return cls(opcode & 7, a & 3, b & 3, c & 3)


class QCPU:
    def __init__(self) -> None:
        self.registers = [0] * 4
        self.memory = [0] * 256
        self.pc = 0
        self.halted = False

    def load_program(self, program: List[QInstruction]) -> None:
        self.memory[: len(program)] = [self._encode_inst(inst) for inst in program]
        self.pc = 0
        self.halted = False

    @staticmethod
    def _encode_inst(inst: QInstruction) -> int:
        return (inst.opcode << 8) | (inst.a << 6) | (inst.b << 4) | (inst.c << 2)

    @staticmethod
    def _decode_inst(word: int) -> QInstruction:
        return QInstruction((word >> 8) & 7, (word >> 6) & 3, (word >> 4) & 3, (word >> 2) & 3)

    def step(self) -> None:
        word = self.memory[self.pc]
        inst = self._decode_inst(word)
        self.pc = (self.pc + 1) % len(self.memory)

        if inst.opcode == 0:  # ADD
            self.registers[inst.c] = (self.registers[inst.a] + self.registers[inst.b]) % BASE
        elif inst.opcode == 1:  # MUL
            self.registers[inst.c] = (self.registers[inst.a] * self.registers[inst.b]) % BASE
        elif inst.opcode == 2:  # LOAD
            self.registers[inst.a] = self.memory[self.registers[inst.b] % len(self.memory)] % BASE
        elif inst.opcode == 3:  # STORE
            self.memory[self.registers[inst.b] % len(self.memory)] = self.registers[inst.a] % BASE
        elif inst.opcode == 4:  # BRZ
            if self.registers[inst.a] == 0:
                self.pc = inst.b
        elif inst.opcode == 5:  # MIN
            self.registers[inst.c] = min(self.registers[inst.a], self.registers[inst.b])
        elif inst.opcode == 6:  # MAX
            self.registers[inst.c] = max(self.registers[inst.a], self.registers[inst.b])
        elif inst.opcode == 7:  # HALT
            self.halted = True
        else:
            raise RuntimeError(f"Unknown opcode: {inst.opcode}")

    def run(self, cycles: int = 256) -> None:
        for _ in range(cycles):
            if self.halted:
                return
            self.step()
        raise RuntimeError("QCPU did not halt in allotted cycles")


def verify_programs() -> None:
    programs: Dict[str, List[QInstruction]] = {
        "add": [
            QInstruction.encode(0, 0, 1, 2),
            QInstruction.encode(7, 0, 0, 0),
        ],
        "mul": [
            QInstruction.encode(1, 0, 1, 2),
            QInstruction.encode(7, 0, 0, 0),
        ],
        "store_load": [
            QInstruction.encode(3, 0, 3, 0),
            QInstruction.encode(2, 2, 3, 0),
            QInstruction.encode(7, 0, 0, 0),
        ],
        "min_max": [
            QInstruction.encode(5, 0, 1, 2),
            QInstruction.encode(6, 0, 1, 3),
            QInstruction.encode(7, 0, 0, 0),
        ],
        "branch_zero": [
            QInstruction.encode(4, 0, 3, 0),
            QInstruction.encode(7, 0, 0, 0),
            QInstruction.encode(0, 0, 1, 2),
            QInstruction.encode(7, 0, 0, 0),
        ],
        "memory_move": [
            QInstruction.encode(3, 0, 3, 0),
            QInstruction.encode(2, 2, 3, 0),
            QInstruction.encode(7, 0, 0, 0),
        ],
    }

    expected = {
        "add": [1, 2, 3, 8],
        "mul": [1, 2, 2, 8],
        "store_load": [1, 2, 1, 8],
        "min_max": [1, 2, 1, 2],
        "branch_zero": [0, 1, 0, 8],
        "memory_move": [1, 2, 1, 8],
    }

    initial_registers = {
        "add": [1, 2, 0, 8],
        "mul": [1, 2, 0, 8],
        "store_load": [1, 2, 0, 8],
        "min_max": [1, 2, 0, 8],
        "branch_zero": [0, 1, 0, 8],
        "memory_move": [1, 2, 0, 8],
    }

    print("QCPU integration: 6 programs")
    for name, program in programs.items():
        cpu = QCPU()
        cpu.registers = initial_registers[name].copy()
        cpu.load_program(program)
        cpu.run(128)

        if cpu.registers[:4] != expected[name]:
            raise AssertionError(f"Program {name} failed: {cpu.registers[:4]} != {expected[name]}")

    print("QCPU integration complete: 6 programs passed.")


if __name__ == "__main__":
    verify_programs()
