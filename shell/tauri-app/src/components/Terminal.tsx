import React, { useEffect, useRef } from "react";
import { Terminal as XTerm } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import { getAiSuggestion } from "../services/ai-suggestions";
import { invoke } from "@tauri-apps/api/tauri";
import "xterm/css/xterm.css";

const Terminal: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<XTerm | null>(null);
  const lineRef = useRef<string>("");

  useEffect(() => {
    if (!containerRef.current) return;

    const term = new XTerm({ cursorBlink: true, theme: { background: "#1e1e2e" } });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(containerRef.current);
    fit.fit();
    termRef.current = term;

    term.writeln("AI OS Shell — type a command or natural language query");
    term.write("$ ");

    term.onKey(async ({ key, domEvent }) => {
      const term = termRef.current!;
      if (domEvent.key === "Enter") {
        const line = lineRef.current.trim();
        lineRef.current = "";
        term.writeln("");
        if (line) {
          const suggestion = await getAiSuggestion(line);
          if (suggestion !== line) {
            term.writeln(`\x1b[33m[AI] Did you mean: ${suggestion}\x1b[0m`);
          }
          const output: string = await invoke("run_command", { command: line });
          term.writeln(output);
        }
        term.write("$ ");
      } else if (domEvent.key === "Backspace") {
        if (lineRef.current.length > 0) {
          lineRef.current = lineRef.current.slice(0, -1);
          term.write("\b \b");
        }
      } else {
        lineRef.current += key;
        term.write(key);
      }
    });

    const handleResize = () => fit.fit();
    window.addEventListener("resize", handleResize);
    return () => {
      term.dispose();
      window.removeEventListener("resize", handleResize);
    };
  }, []);

  return <div ref={containerRef} style={{ width: "100%", height: "100%" }} />;
};

export default Terminal;
