import { invoke } from "@tauri-apps/api/tauri";

/**
 * Ask the command interpreter model for the canonical form of a raw
 * user input string (shell command, natural language, etc.).
 */
export async function getAiSuggestion(rawInput: string): Promise<string> {
  try {
    const result = await invoke<{ intent: string; structured_json: string; confidence: number }>(
      "interpret_command",
      { rawCommand: rawInput }
    );
    if (result.confidence > 0.7) {
      return result.intent;
    }
  } catch {
    // If the backend is unavailable, fall back to the original input.
  }
  return rawInput;
}
