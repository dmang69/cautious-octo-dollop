# Model Training Guide

## Scheduler Model

### 1. Collect Training Data

```bash
python models/scheduler/data_collection.py \
    --endpoint http://127.0.0.1:50051 \
    --duration 3600 \
    --output models/scheduler/data/telemetry.npz
```

### 2. Train

```bash
cd models/scheduler
pip install -r requirements.txt
python train.py --data data/telemetry.npz \
                --output ../../models/pretrained/scheduler_v1.onnx \
                --epochs 100
```

## Command Interpreter Model

### 1. Prepare Data

Create a JSON file `models/command-interpreter/data/commands.json`:

```json
[
  {"input": "show me running processes", "intent": "list_processes"},
  {"input": "kill process 1234",         "intent": "kill_process"},
  {"input": "how much memory is free",   "intent": "memory_stats"}
]
```

### 2. Fine-tune

```bash
cd models/command-interpreter
python fine_tune_llm.py \
    --data data/commands.json \
    --output-dir checkpoints/ \
    --onnx ../../models/pretrained/command_interpreter_v1.onnx
```

## Updating the Registry

After exporting a model, update `models/pretrained/model_registry.json` with the new SHA-256 hash:

```bash
sha256sum models/pretrained/scheduler_v1.onnx
```
