#!/usr/bin/env python3
"""QCPU integration tests using an 8-quat instruction model."""

BASE = 4
WORD_QUATS = 4
MODULUS = BASE ** WORD_QUATS


def int_to_quats(value, width=WORD_QUATS):
    quats = []
    for _ in range(width):
        quats.append(value % BASE)
        value //= BASE
    return quats


def quats_to_int(quats):
    value = 0
    for index, quat in enumerate(quats):
        value += quat * (BASE ** index)
    return value


def q_tsum_int(a, b):
    qa = int_to_quats(a)
    qb = int_to_quats(b)
    return quats_to_int([(x + y) % BASE for x, y in zip(qa, qb)])


class QCPU:
    def __init__(self):
        self.registers = {f"r{index}": 0 for index in range(4)}
        self.memory = {}

    def execute(self, program):
        pc = 0
        while pc < len(program):
            instruction = program[pc]
            op = instruction[0]
            if op == "LOADI":
                _, reg, value = instruction
                self.registers[reg] = value % MODULUS
            elif op == "ADD":
                _, dest, src = instruction
                self.registers[dest] = (self.registers[dest] + self.registers[src]) % MODULUS
            elif op == "SUB":
                _, dest, src = instruction
                self.registers[dest] = (self.registers[dest] - self.registers[src]) % MODULUS
            elif op == "MUL":
                _, dest, src = instruction
                self.registers[dest] = (self.registers[dest] * self.registers[src]) % MODULUS
            elif op == "MIN":
                _, dest, src = instruction
                self.registers[dest] = min(self.registers[dest], self.registers[src])
            elif op == "MAX":
                _, dest, src = instruction
                self.registers[dest] = max(self.registers[dest], self.registers[src])
            elif op == "TSUM":
                _, dest, src = instruction
                self.registers[dest] = q_tsum_int(self.registers[dest], self.registers[src])
            elif op == "STORE":
                _, reg, addr = instruction
                self.memory[addr] = self.registers[reg]
            elif op == "LOAD":
                _, reg, addr = instruction
                self.registers[reg] = self.memory.get(addr, 0)
            elif op == "HALT":
                break
            else:
                raise ValueError(f"Unknown opcode: {op}")
            pc += 1


def run_program(name, program, expected):
    cpu = QCPU()
    cpu.execute(program)
    for reg, value in expected.items():
        assert cpu.registers[reg] == value, f"{name} expected {reg}={value}, got {cpu.registers[reg]}"


def run_tests():
    programs = [
        (
            "addition",
            [
                ("LOADI", "r0", 7),
                ("LOADI", "r1", 9),
                ("ADD", "r0", "r1"),
                ("HALT",),
            ],
            {"r0": 16},
        ),
        (
            "subtraction",
            [
                ("LOADI", "r0", 7),
                ("LOADI", "r1", 9),
                ("SUB", "r0", "r1"),
                ("HALT",),
            ],
            {"r0": (7 - 9) % MODULUS},
        ),
        (
            "factorial",
            [
                ("LOADI", "r0", 1),
                ("LOADI", "r1", 2),
                ("MUL", "r0", "r1"),
                ("LOADI", "r1", 3),
                ("MUL", "r0", "r1"),
                ("LOADI", "r1", 4),
                ("MUL", "r0", "r1"),
                ("LOADI", "r1", 5),
                ("MUL", "r0", "r1"),
                ("HALT",),
            ],
            {"r0": 120},
        ),
        (
            "min_max",
            [
                ("LOADI", "r0", 42),
                ("LOADI", "r1", 17),
                ("MIN", "r0", "r1"),
                ("LOADI", "r2", 99),
                ("MAX", "r2", "r1"),
                ("HALT",),
            ],
            {"r0": 17, "r2": 99},
        ),
        (
            "tsum",
            [
                ("LOADI", "r0", 200),
                ("LOADI", "r1", 100),
                ("TSUM", "r0", "r1"),
                ("HALT",),
            ],
            {"r0": q_tsum_int(200, 100)},
        ),
        (
            "memory",
            [
                ("LOADI", "r0", 42),
                ("STORE", "r0", 3),
                ("LOADI", "r1", 0),
                ("LOAD", "r1", 3),
                ("ADD", "r1", "r0"),
                ("HALT",),
            ],
            {"r1": 84},
        ),
    ]

    for name, program, expected in programs:
        run_program(name, program, expected)

    print(f"[QCPU] Verified {len(programs)} integration programs.")


if __name__ == "__main__":
    run_tests()
