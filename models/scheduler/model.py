"""Neural network architecture for the process scheduler model."""

import torch
import torch.nn as nn


class SchedulerNet(nn.Module):
    """Feed-forward network mapping telemetry vectors to priority adjustments.

    Input:  (batch, input_dim)  — telemetry features per process
    Output: (batch, output_dim) — suggested priority delta per process
    """

    def __init__(self, input_dim: int = 16, output_dim: int = 1, hidden: int = 128) -> None:
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(input_dim, hidden),
            nn.ReLU(),
            nn.LayerNorm(hidden),
            nn.Linear(hidden, hidden),
            nn.ReLU(),
            nn.LayerNorm(hidden),
            nn.Linear(hidden, output_dim),
            nn.Tanh(),  # output in [-1, 1], scaled to priority range externally
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.net(x)
