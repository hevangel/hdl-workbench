import type * as Monaco from "monaco-editor";
import type { Diagnostic } from "@xezim-ide/protocol";

type MonacoApi = typeof Monaco;

let languageRegistered = false;

export function registerSystemVerilog(monaco: MonacoApi): void {
  if (!languageRegistered) {
    monaco.languages.register({ id: "systemverilog" });
    monaco.languages.setMonarchTokensProvider("systemverilog", {
      keywords: [
        "always_comb", "always_ff", "always_latch", "assign", "begin", "case", "default", "else", "end",
        "endcase", "endfunction", "endmodule", "endpackage", "endtask", "for", "function", "generate", "if",
        "input", "inout", "integer", "logic", "module", "output", "package", "parameter", "posedge", "negedge",
        "reg", "return", "task", "time", "typedef", "wire",
      ],
      tokenizer: {
        root: [
          [/\/\/.*$/, "comment"],
          [/\/\*/, "comment", "@comment"],
          [/\$[a-zA-Z_]\w*/, "predefined"],
          [/`[a-zA-Z_]\w*/, "macro"],
          [/[a-zA-Z_$][\w$]*/, { cases: { "@keywords": "keyword", "@default": "identifier" } }],
          [/\d+'[sS]?[bBoOdDhH][0-9a-fA-F_xXzZ?]+/, "number"],
          [/\d+/, "number"],
          [/[{}()[\]]/, "@brackets"],
          [/[;,.]/, "delimiter"],
          [/[=<>!~?:&|+\-*\/%^]+/, "operator"],
        ],
        comment: [
          [/[^*/]+/, "comment"],
          [/\*\//, "comment", "@pop"],
          [/[*/]/, "comment"],
        ],
      },
    });
    languageRegistered = true;
  }

  monaco.editor.defineTheme("xezim-studio-dark", {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "keyword", foreground: "C792EA" },
      { token: "macro", foreground: "F78C6C" },
      { token: "predefined", foreground: "82AAFF" },
      { token: "number", foreground: "F78C6C" },
    ],
    colors: {
      "editor.background": "#101319",
      "editorLineNumber.foreground": "#555d6d",
      "editorLineNumber.activeForeground": "#d8deea",
      "editor.selectionBackground": "#27354a",
      "editorCursor.foreground": "#76d6c2",
      "editorLineHighlightBackground": "#171c24",
    },
  });
}

export function toMonacoMarkers(monaco: MonacoApi, diagnostics: Diagnostic[]) {
  return diagnostics.map((diagnostic) => {
    const line = diagnostic.line ?? 1;
    const column = diagnostic.column ?? 1;
    return {
      severity: diagnostic.severity === "error"
        ? monaco.MarkerSeverity.Error
        : diagnostic.severity === "warning"
          ? monaco.MarkerSeverity.Warning
          : monaco.MarkerSeverity.Info,
      message: `${diagnostic.code}: ${diagnostic.message}`,
      startLineNumber: line,
      startColumn: column,
      endLineNumber: line,
      endColumn: column + 1,
      source: "Xezim Studio",
      code: diagnostic.code,
    };
  });
}
