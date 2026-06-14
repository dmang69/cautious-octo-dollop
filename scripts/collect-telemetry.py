#!/usr/bin/env python3
"""Collect system telemetry from the AI Runtime and save for model training."""

import argparse
import json
import time
from pathlib import Path

# grpc stubs would normally be imported from the generated package.
# from ipc.proto import ai_os_services_pb2_grpc as grpc_stub
# from ipc.proto import ai_os_services_pb2 as pb

import numpy as np


def collect(endpoint: str, duration_s: int, output_path: str) -> None:
    records = []
    print(f"Collecting telemetry from {endpoint} for {duration_s}s ...")
    deadline = time.time() + duration_s

    # TODO: replace with live gRPC stream:
    # channel = grpc.insecure_channel(endpoint)
    # stub = grpc_stub.TelemetryServiceStub(channel)
    # for snap in stub.StreamTelemetry(pb.TelemetryRequest()):
    #     records.append(snapshot_to_features(snap))
    #     if time.time() > deadline:
    #         break

    while time.time() < deadline:
        record = {
            "timestamp_ms": int(time.time() * 1000),
            "cpu_avg": float(np.random.rand()),
            "memory_used_bytes": int(np.random.randint(1_000_000, 8_000_000_000)),
            "process_count": int(np.random.randint(50, 500)),
        }
        records.append(record)
        time.sleep(0.5)

    Path(output_path).parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(records, f, indent=2)
    print(f"Saved {len(records)} records to {output_path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Collect AI OS telemetry")
    parser.add_argument("--endpoint", default="http://127.0.0.1:50051")
    parser.add_argument("--duration", type=int, default=60)
    parser.add_argument("--output", default="data/telemetry.json")
    args = parser.parse_args()
    collect(args.endpoint, args.duration, args.output)
