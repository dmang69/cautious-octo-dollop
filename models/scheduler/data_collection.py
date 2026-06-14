"""Collect telemetry from the AI Runtime daemon and save as NumPy archives."""

import argparse
import json
import time
from pathlib import Path

import grpc
import numpy as np

# Generated stubs would be imported here; using placeholder for now.
# from ipc.proto import ai_os_services_pb2 as pb
# from ipc.proto import ai_os_services_pb2_grpc as grpc_stub


def collect(endpoint: str, duration_s: int, output_path: str) -> None:
    records = []
    deadline = time.time() + duration_s
    print(f"Collecting telemetry from {endpoint} for {duration_s}s ...")

    # TODO: connect to gRPC TelemetryService.StreamTelemetry
    # channel = grpc.insecure_channel(endpoint)
    # stub = grpc_stub.TelemetryServiceStub(channel)
    # for snapshot in stub.StreamTelemetry(pb.TelemetryRequest()):
    #     records.append(snapshot_to_array(snapshot))
    #     if time.time() > deadline:
    #         break

    # Placeholder: generate synthetic data
    while time.time() < deadline:
        records.append(np.random.rand(16).tolist())
        time.sleep(0.5)

    X = np.array(records, dtype=np.float32)
    y = np.zeros((len(X), 1), dtype=np.float32)  # labels filled during labelling step
    Path(output_path).parent.mkdir(parents=True, exist_ok=True)
    np.savez(output_path, X=X, y=y)
    print(f"Saved {len(X)} samples to {output_path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--endpoint", default="http://127.0.0.1:50051")
    parser.add_argument("--duration", type=int, default=60)
    parser.add_argument("--output", default="data/telemetry.npz")
    args = parser.parse_args()
    collect(args.endpoint, args.duration, args.output)
