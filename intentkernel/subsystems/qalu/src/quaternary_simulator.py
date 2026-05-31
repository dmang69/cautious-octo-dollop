#!/usr/bin/env python3
"""Q-ALU quaternary arithmetic verification."""

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


def q_add(a, b):
    res = []
    carry = 0
    for qa, qb in zip(a, b):
        total = qa + qb + carry
        res.append(total % BASE)
        carry = total // BASE
    return res


def q_sub(a, b):
    res = []
    borrow = 0
    for qa, qb in zip(a, b):
        total = qa - qb - borrow
        if total < 0:
            total += BASE
            borrow = 1
        else:
            borrow = 0
        res.append(total)
    return res


def q_mul(a, b):
    return int_to_quats((quats_to_int(a) * quats_to_int(b)) % MODULUS)


def q_min(a, b):
    return [min(qa, qb) for qa, qb in zip(a, b)]


def q_max(a, b):
    return [max(qa, qb) for qa, qb in zip(a, b)]


def q_tsum(a, b):
    return [(qa + qb) % BASE for qa, qb in zip(a, b)]


def q_and(a, b):
    return [qa & qb for qa, qb in zip(a, b)]


def q_or(a, b):
    return [qa | qb for qa, qb in zip(a, b)]


def q_xor(a, b):
    return [qa ^ qb for qa, qb in zip(a, b)]


def q_inc(a):
    return int_to_quats((quats_to_int(a) + 1) % MODULUS)


def q_dec(a):
    return int_to_quats((quats_to_int(a) - 1) % MODULUS)


def run_tests():
    cases = 0
    assertions = 0
    for a in range(MODULUS):
        qa = int_to_quats(a)
        assert quats_to_int(q_inc(qa)) == (a + 1) % MODULUS
        assert quats_to_int(q_dec(qa)) == (a - 1) % MODULUS
        assertions += 2
        for b in range(MODULUS):
            qb = int_to_quats(b)
            cases += 1
            assert quats_to_int(q_add(qa, qb)) == (a + b) % MODULUS
            assert quats_to_int(q_sub(qa, qb)) == (a - b) % MODULUS
            assert quats_to_int(q_mul(qa, qb)) == (a * b) % MODULUS
            assert q_min(qa, qb) == [min(x, y) for x, y in zip(qa, qb)]
            assert q_max(qa, qb) == [max(x, y) for x, y in zip(qa, qb)]
            assert q_tsum(qa, qb) == [(x + y) % BASE for x, y in zip(qa, qb)]
            assert q_and(qa, qb) == [x & y for x, y in zip(qa, qb)]
            assert q_or(qa, qb) == [x | y for x, y in zip(qa, qb)]
            assert q_xor(qa, qb) == [x ^ y for x, y in zip(qa, qb)]
            assertions += 9

    print(f"[Q-ALU] Verified {cases} cases with {assertions} assertions.")


if __name__ == "__main__":
    run_tests()
