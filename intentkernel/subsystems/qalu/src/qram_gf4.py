#!/usr/bin/env python3
"""Q-RAM GF(4) [5,3] ECC verification."""

GF4_MUL = (
    (0, 0, 0, 0),
    (0, 1, 2, 3),
    (0, 2, 3, 1),
    (0, 3, 1, 2),
)


def gf4_add(a, b):
    return a ^ b


def gf4_mul(a, b):
    return GF4_MUL[a][b]


def encode(message):
    m0, m1, m2 = message
    p0 = gf4_add(gf4_add(m0, m1), m2)
    p1 = gf4_add(gf4_add(m0, gf4_mul(2, m1)), gf4_mul(3, m2))
    return [m0, m1, m2, p0, p1]


def hamming_distance(a, b):
    return sum(1 for left, right in zip(a, b) if left != right)


def decode(word, codebook):
    best = None
    best_distance = 6
    for codeword, message in codebook:
        distance = hamming_distance(codeword, word)
        if distance < best_distance:
            best_distance = distance
            best = message
            if best_distance == 0:
                break
    if best_distance > 1:
        raise ValueError("Uncorrectable error")
    return list(best), best_distance


def build_codebook():
    codebook = []
    for m0 in range(4):
        for m1 in range(4):
            for m2 in range(4):
                message = (m0, m1, m2)
                codebook.append((encode(message), message))
    return codebook


def run_tests():
    codebook = build_codebook()
    cases = 0
    for message in codebook:
        codeword = encode(message[1])
        for position in range(len(codeword)):
            for error in (1, 2, 3):
                corrupted = list(codeword)
                corrupted[position] = gf4_add(corrupted[position], error)
                decoded, distance = decode(corrupted, codebook)
                assert decoded == list(message[1])
                assert distance <= 1
                cases += 1

    print(f"[Q-RAM] Verified {cases} single-symbol corrections.")


if __name__ == "__main__":
    run_tests()
