//! IntentOS — WASM Intent Parser
//!
//! Reads a JSON request from stdin, classifies the user's intent, and writes
//! a JSON response to stdout.  Compiled to wasm32-wasip1 for sandboxed
//! execution inside the kernel-interface layer.
//!
//! Input  (JSON):  { "text": "<user prompt>" }
//! Output (JSON):  { "intent": "<kind>", "confidence": <0.0–1.0>, "params": { … } }

use std::collections::HashMap;
use std::io::{self, Read, Write};

// ---------------------------------------------------------------------------
// Minimal JSON helpers (no external crates — keeps the WASM binary small)
// ---------------------------------------------------------------------------

fn json_string(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{}\"", escaped)
}

fn json_f64(v: f64) -> String {
    format!("{:.4}", v)
}

fn build_json_object(fields: &[(&str, String)]) -> String {
    let inner: Vec<String> = fields
        .iter()
        .map(|(k, v)| format!("{}: {}", json_string(k), v))
        .collect();
    format!("{{{}}}", inner.join(", "))
}

/// Very small JSON string extractor — only handles flat `"key": "value"` pairs.
fn extract_string_field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{}\"", key);
    let pos = json.find(&needle)?;
    let after_key = &json[pos + needle.len()..];
    let colon = after_key.find(':')? + 1;
    let rest = after_key[colon..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let content = &rest[1..];
    let end = content.find('"')?;
    Some(&content[..end])
}

// ---------------------------------------------------------------------------
// Intent classification
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
enum Intent {
    Run,
    Query,
    Configure,
    List,
    Help,
    Unknown,
}

impl Intent {
    fn as_str(&self) -> &'static str {
        match self {
            Intent::Run       => "run",
            Intent::Query     => "query",
            Intent::Configure => "configure",
            Intent::List      => "list",
            Intent::Help      => "help",
            Intent::Unknown   => "unknown",
        }
    }
}

struct ClassificationResult {
    intent:     Intent,
    confidence: f64,
    params:     HashMap<&'static str, String>,
}

/// Keyword-based intent classifier.
fn classify(text: &str) -> ClassificationResult {
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    // Each entry: (intent, trigger keywords, boost keywords)
    let rules: &[(Intent, &[&str], &[&str])] = &[
        (
            Intent::Run,
            &["run", "execute", "start", "launch", "trigger", "invoke"],
            &["agent", "task", "job", "pipeline", "workflow"],
        ),
        (
            Intent::Query,
            &["what", "show", "get", "fetch", "status", "check", "how", "tell"],
            &["is", "are", "running", "state", "health", "info", "result"],
        ),
        (
            Intent::Configure,
            &["set", "configure", "update", "change", "enable", "disable", "toggle"],
            &["config", "setting", "option", "parameter", "flag"],
        ),
        (
            Intent::List,
            &["list", "ls", "all", "show all", "enumerate"],
            &["agents", "skills", "runs", "tasks", "plugins"],
        ),
        (
            Intent::Help,
            &["help", "?", "usage", "how to", "howto", "guide", "docs"],
            &["command", "syntax", "example", "tutorial"],
        ),
    ];

    let mut best_intent    = Intent::Unknown;
    let mut best_score     = 0u32;

    for (intent, triggers, boosts) in rules {
        let mut score = 0u32;
        for &t in *triggers {
            if words.contains(&t) || lower.contains(t) {
                score += 2;
            }
        }
        for &b in *boosts {
            if words.contains(&b) || lower.contains(b) {
                score += 1;
            }
        }
        if score > best_score {
            best_score  = score;
            best_intent = match intent {
                Intent::Run       => Intent::Run,
                Intent::Query     => Intent::Query,
                Intent::Configure => Intent::Configure,
                Intent::List      => Intent::List,
                Intent::Help      => Intent::Help,
                Intent::Unknown   => Intent::Unknown,
            };
        }
    }

    // Map raw score to a 0–1 confidence value.
    let confidence = if best_score == 0 {
        0.0
    } else {
        let raw = best_score as f64 / 6.0; // 6 = typical saturating score
        raw.min(1.0)
    };

    // Extract lightweight params from the text.
    let mut params: HashMap<&'static str, String> = HashMap::new();
    params.insert("raw_text", text.to_owned());

    // Try to pull a quoted target from the prompt, e.g. run "planner"
    if let Some(start) = text.find('"') {
        if let Some(end) = text[start + 1..].find('"') {
            params.insert("target", text[start + 1..start + 1 + end].to_owned());
        }
    }

    ClassificationResult {
        intent: best_intent,
        confidence,
        params,
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    // Read all of stdin.
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        let err = build_json_object(&[
            ("error", json_string("failed to read stdin")),
        ]);
        let _ = io::stdout().write_all(err.as_bytes());
        return;
    }

    let text = match extract_string_field(&input, "text") {
        Some(t) => t.to_owned(),
        None => {
            let err = build_json_object(&[
                ("error", json_string("missing 'text' field in input JSON")),
            ]);
            let _ = io::stdout().write_all(err.as_bytes());
            return;
        }
    };

    let result = classify(&text);

    // Build params JSON object.
    let params_fields: Vec<(&str, String)> = result
        .params
        .iter()
        .map(|(k, v)| (*k, json_string(v)))
        .collect();
    let params_json = build_json_object(&params_fields);

    let output = build_json_object(&[
        ("intent",     json_string(result.intent.as_str())),
        ("confidence", json_f64(result.confidence)),
        ("params",     params_json),
    ]);

    let _ = io::stdout().write_all(output.as_bytes());
    let _ = io::stdout().write_all(b"\n");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_intent() {
        let r = classify("run agent planner");
        assert_eq!(r.intent, Intent::Run);
        assert!(r.confidence > 0.0);
    }

    #[test]
    fn query_intent() {
        let r = classify("what is the status");
        assert_eq!(r.intent, Intent::Query);
    }

    #[test]
    fn configure_intent() {
        let r = classify("set config debug to true");
        assert_eq!(r.intent, Intent::Configure);
    }

    #[test]
    fn list_intent() {
        let r = classify("list all agents");
        assert_eq!(r.intent, Intent::List);
    }

    #[test]
    fn help_intent() {
        let r = classify("help how to run a task");
        assert_eq!(r.intent, Intent::Help);
    }

    #[test]
    fn unknown_intent() {
        let r = classify("xyzzy frobnosticate");
        assert_eq!(r.intent, Intent::Unknown);
        assert_eq!(r.confidence, 0.0);
    }

    #[test]
    fn target_extraction() {
        let r = classify("run \"planner\" agent");
        assert_eq!(r.params.get("target").map(String::as_str), Some("planner"));
    }
}
