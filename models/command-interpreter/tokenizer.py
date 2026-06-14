"""Tokenizer for the command interpreter model."""

import re
from typing import List


VOCAB_FILE = "vocab.txt"


class CommandTokenizer:
    """Simple whitespace tokenizer with a fixed vocabulary.

    For production use, replace with a BPE tokenizer (e.g., sentencepiece).
    """

    def __init__(self, vocab_path: str = VOCAB_FILE) -> None:
        try:
            with open(vocab_path) as f:
                tokens = [line.strip() for line in f if line.strip()]
        except FileNotFoundError:
            tokens = []
        self.token2id = {tok: i + 4 for i, tok in enumerate(tokens)}
        self.token2id.update({"<pad>": 0, "<unk>": 1, "<bos>": 2, "<eos>": 3})
        self.id2token = {v: k for k, v in self.token2id.items()}

    def encode(self, text: str) -> List[int]:
        tokens = re.split(r"\s+", text.strip().lower())
        return [2] + [self.token2id.get(t, 1) for t in tokens] + [3]

    def decode(self, ids: List[int]) -> str:
        return " ".join(
            self.id2token.get(i, "<unk>") for i in ids if i not in (0, 2, 3)
        )
