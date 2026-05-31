// qalu_cap.rs — Quaternary ALU capability wrapper
// Exposes Q-ALU operations as IntentKernel capability tokens
// Each operation is a bounded, revocable capability

use std::process::Command;

pub enum QAluOp {
    Add,
    Sub,
    Mul,
    Min,
    Max,
    Xor,
}

pub struct QAluCapability {
    pub op: QAluOp,
    pub width_qits: u8,
    pub caller_cap: u64,
}

pub fn invoke_qalu(cap: QAluCapability, a: u64, b: u64) -> Result<u64, String> {
    let op_name = match cap.op {
        QAluOp::Add => "ADD",
        QAluOp::Sub => "SUB",
        QAluOp::Mul => "MUL",
        QAluOp::Min => "MIN",
        QAluOp::Max => "MAX",
        QAluOp::Xor => "XOR",
    };

    let script = format!(
        r#"import importlib.util, os, sys
root = os.getcwd()
module_path = os.path.join(root, 'subsystems', 'qalu', 'src', 'qalu.py')
spec = importlib.util.spec_from_file_location('qalu', module_path)
qalu = importlib.util.module_from_spec(spec)
sys.modules['qalu'] = qualu
spec.loader.exec_module(qualu)
QALU = qualu.QALU
QWord = qualu.QWord
alu = QALU(width={})
a = QWord.from_int({}, {})
b = QWord.from_int({}, {})
print(getattr(alu, '{op_name}')(a, b).to_int())"#,
        cap.width_qits,
        a,
        cap.width_qits,
        b,
        cap.width_qits,
        op_name = op_name,
    );

    let out = Command::new("python3")
        .arg("-c")
        .arg(&script)
        .output()
        .map_err(|e| e.to_string())?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }

    let result_text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let result = result_text
        .parse::<u64>()
        .map_err(|e| format!("failed to parse Q-ALU output: {}", e))?;
    Ok(result)
}
