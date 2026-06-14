"""Reinforcement learning training pipeline for the process scheduler model."""

import argparse
import numpy as np
import torch
from torch.utils.data import DataLoader, TensorDataset

from model import SchedulerNet


def train(data_path: str, output_path: str, epochs: int = 50) -> None:
    data = np.load(data_path)
    X = torch.tensor(data["X"], dtype=torch.float32)
    y = torch.tensor(data["y"], dtype=torch.float32)

    dataset = TensorDataset(X, y)
    loader = DataLoader(dataset, batch_size=256, shuffle=True)

    model = SchedulerNet(input_dim=X.shape[1], output_dim=y.shape[1])
    optimizer = torch.optim.Adam(model.parameters(), lr=1e-3)
    criterion = torch.nn.MSELoss()

    for epoch in range(1, epochs + 1):
        total_loss = 0.0
        for xb, yb in loader:
            optimizer.zero_grad()
            loss = criterion(model(xb), yb)
            loss.backward()
            optimizer.step()
            total_loss += loss.item()
        print(f"Epoch {epoch}/{epochs}  loss={total_loss / len(loader):.4f}")

    # Export to ONNX
    dummy = torch.zeros(1, X.shape[1])
    torch.onnx.export(
        model,
        dummy,
        output_path,
        input_names=["telemetry"],
        output_names=["priorities"],
        opset_version=17,
    )
    print(f"Model exported to {output_path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--data", default="data/telemetry.npz")
    parser.add_argument("--output", default="../../models/pretrained/scheduler_v1.onnx")
    parser.add_argument("--epochs", type=int, default=50)
    args = parser.parse_args()
    train(args.data, args.output, args.epochs)
