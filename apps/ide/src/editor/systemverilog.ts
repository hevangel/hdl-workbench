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

  monaco.editor.defineTheme("hdl-workbench-dark", {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "keyword", foreground: "C792EA" },
      { token: "macro", foreground: "F78C6C" },
      { token: "predefined", foreground: "82AAFF" },
      { token: "number", foreground: "F78C6C" },
      { token: "comment", foreground: "5F6877", fontStyle: "italic" },
    ],
    colors: {
      "editor.background": "#0e1116",
      "editorGutter.background": "#0e1116",
      "editorLineNumber.foreground": "#4b5361",
      "editorLineNumber.activeForeground": "#c9d1dc",
      "editor.selectionBackground": "#264a46",
      "editor.inactiveSelectionBackground": "#1d2a2c",
      "editorCursor.foreground": "#5fd4b8",
      "editor.lineHighlightBackground": "#151920",
      "editor.lineHighlightBorder": "#00000000",
      "editorIndentGuide.background1": "#1c2129",
      "editorWidget.background": "#161a21",
      "editorWidget.border": "#2a303a",
      "scrollbarSlider.background": "#2a303a80",
      "scrollbarSlider.hoverBackground": "#3a414d99",
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
      source: "HDL Workbench",
      code: diagnostic.code,
    };
  });
}
