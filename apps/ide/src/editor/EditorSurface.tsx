import { useEffect, useRef, useState } from "react";
import Editor, { type BeforeMount, type OnMount } from "@monaco-editor/react";
import type { Diagnostic, ProjectPath } from "@xezim-ide/protocol";
import type * as Monaco from "monaco-editor";
import { registerSystemVerilog, toMonacoMarkers } from "./systemverilog.js";

interface EditorSurfaceProps {
  path: ProjectPath | null;
  value: string;
  diagnostics: Diagnostic[];
  onChange(value: string): void;
  readOnly?: boolean;
}

export function EditorSurface({ path, value, diagnostics, onChange, readOnly = false }: EditorSurfaceProps) {
  const editorRef = useRef<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const [monacoApi, setMonacoApi] = useState<typeof Monaco | null>(null);

  // The theme must exist before Monaco creates the editor, or it silently falls back to the light theme.
  const handleBeforeMount: BeforeMount = (monaco) => registerSystemVerilog(monaco);

  const handleMount: OnMount = (editor, monaco) => {
    registerSystemVerilog(monaco);
    editorRef.current = editor;
    setMonacoApi(monaco);
  };

  const language = path?.toLowerCase().endsWith(".c") ? "c"
    : path?.toLowerCase().endsWith(".cc") || path?.toLowerCase().endsWith(".cpp") || path?.toLowerCase().endsWith(".cxx") ? "cpp"
      : path?.toLowerCase().endsWith(".h") ? "c"
      : path?.toLowerCase().endsWith(".hpp") || path?.toLowerCase().endsWith(".hh") || path?.toLowerCase().endsWith(".hxx") ? "cpp"
        : path?.toLowerCase().endsWith(".inc") ? "c"
          : "systemverilog";

  useEffect(() => {
    const model = editorRef.current?.getModel();
    if (!model || !monacoApi) return;
    const relevant = diagnostics.filter((item) => !item.path || item.path === path);
    monacoApi.editor.setModelMarkers(model, "hdl-workbench", toMonacoMarkers(monacoApi, relevant));
  }, [diagnostics, monacoApi, path]);

  if (!path) {
    return <div className="editor-empty">Choose a SystemVerilog file to start editing.</div>;
  }

  return (
    <div className="editor-canvas" data-testid="editor-surface">
      <Editor
        height="100%"
        language={language}
        theme="hdl-workbench-dark"
        path={path}
        value={value}
        onChange={(nextValue) => onChange(nextValue ?? "")}
        beforeMount={handleBeforeMount}
        onMount={handleMount}
        options={{
          ariaLabel: `${language === "systemverilog" ? "SystemVerilog" : language === "c" ? "C" : "C++"} editor`,
          automaticLayout: true,
          readOnly,
          fontFamily: "'Cascadia Code', 'SFMono-Regular', Consolas, monospace",
          fontSize: 13,
          lineHeight: 21,
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          tabSize: 2,
          wordWrap: "on",
          padding: { top: 12, bottom: 16 },
          renderLineHighlight: "all",
          smoothScrolling: true,
          cursorBlinking: "smooth",
          scrollbar: { verticalScrollbarSize: 10, horizontalScrollbarSize: 10 },
        }}
      />
    </div>
  );
}
