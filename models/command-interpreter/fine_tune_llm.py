"""Fine-tune a small LLM on command intent data and export to ONNX."""

import argparse
import json
from pathlib import Path

import torch
from transformers import (
    AutoModelForSeq2SeqLM,
    AutoTokenizer,
    Seq2SeqTrainer,
    Seq2SeqTrainingArguments,
    DataCollatorForSeq2Seq,
)
from datasets import Dataset


MODEL_NAME = "google/flan-t5-small"


def load_dataset(path: str) -> Dataset:
    with open(path) as f:
        records = json.load(f)
    return Dataset.from_list(records)


def fine_tune(data_path: str, output_dir: str, onnx_path: str) -> None:
    tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME)
    model = AutoModelForSeq2SeqLM.from_pretrained(MODEL_NAME)

    dataset = load_dataset(data_path)

    def preprocess(batch):
        enc = tokenizer(batch["input"], truncation=True, padding="max_length", max_length=64)
        labels = tokenizer(batch["intent"], truncation=True, padding="max_length", max_length=32)
        enc["labels"] = labels["input_ids"]
        return enc

    dataset = dataset.map(preprocess, batched=True)

    args = Seq2SeqTrainingArguments(
        output_dir=output_dir,
        num_train_epochs=3,
        per_device_train_batch_size=16,
        predict_with_generate=True,
        fp16=torch.cuda.is_available(),
        save_strategy="epoch",
        logging_steps=50,
    )

    trainer = Seq2SeqTrainer(
        model=model,
        args=args,
        train_dataset=dataset,
        data_collator=DataCollatorForSeq2Seq(tokenizer, model=model),
    )
    trainer.train()

    # Export to ONNX
    Path(onnx_path).parent.mkdir(parents=True, exist_ok=True)
    dummy_ids = torch.zeros(1, 64, dtype=torch.long)
    torch.onnx.export(
        model,
        (dummy_ids, dummy_ids),
        onnx_path,
        input_names=["input_ids", "attention_mask"],
        output_names=["logits"],
        opset_version=17,
    )
    print(f"Exported to {onnx_path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--data", default="data/commands.json")
    parser.add_argument("--output-dir", default="checkpoints/")
    parser.add_argument("--onnx", default="../../models/pretrained/command_interpreter_v1.onnx")
    args = parser.parse_args()
    fine_tune(args.data, args.output_dir, args.onnx)
