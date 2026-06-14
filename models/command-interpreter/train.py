"""Training pipeline for the NLP command interpreter model."""

import argparse
from pathlib import Path

import torch
from torch.utils.data import DataLoader, Dataset
from transformers import AutoModelForSeq2SeqLM, AutoTokenizer

from tokenizer import CommandTokenizer


class CommandDataset(Dataset):
    def __init__(self, samples: list[dict], tokenizer: CommandTokenizer) -> None:
        self.samples = samples
        self.tokenizer = tokenizer

    def __len__(self) -> int:
        return len(self.samples)

    def __getitem__(self, idx: int) -> dict:
        sample = self.samples[idx]
        enc = self.tokenizer.encode(sample["input"])
        return {"input_ids": torch.tensor(enc), "label": sample["intent"]}


def train(data_path: str, output_path: str, epochs: int = 3) -> None:
    import json

    with open(data_path) as f:
        samples = json.load(f)

    tokenizer = CommandTokenizer()
    dataset = CommandDataset(samples, tokenizer)
    loader = DataLoader(dataset, batch_size=16, shuffle=True)

    print(f"Training on {len(dataset)} samples for {epochs} epoch(s)")
    # Full fine-tuning implemented in fine_tune_llm.py
    print(f"Model would be exported to {output_path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--data", default="data/commands.json")
    parser.add_argument("--output", default="../../models/pretrained/command_interpreter_v1.onnx")
    parser.add_argument("--epochs", type=int, default=3)
    args = parser.parse_args()
    train(args.data, args.output, args.epochs)
