import { useEffect, useRef, useState } from "react";
import Editor, { type OnMount } from "@monaco-editor/react";
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

  const handleMount: OnMount = (editor, monaco) => {
    registerSystemVerilog(monaco);
    editorRef.current = editor;
    setMonacoApi(monaco);
  };

  useEffect(() => {
    const model = editorRef.current?.getModel();
    if (!model || !monacoApi) return;
    const relevant = diagnostics.filter((item) => !item.path || item.path === path);
    monacoApi.editor.setModelMarkers(model, "xezim-studio", toMonacoMarkers(monacoApi, relevant));
  }, [diagnostics, monacoApi, path]);

  if (!path) {
    return <div className="editor-empty">Choose a SystemVerilog file to start editing.</div>;
  }

  return (
    <div className="editor-canvas" data-testid="editor-surface">
      <Editor
        height="100%"
        language="systemverilog"
        theme="xezim-studio-dark"
        path={path}
        value={value}
        onChange={(nextValue) => onChange(nextValue ?? "")}
        onMount={handleMount}
        options={{
          ariaLabel: "SystemVerilog editor",
          automaticLayout: true,
          readOnly,
          fontFamily: "'Cascadia Code', 'SFMono-Regular', Consolas, monospace",
          fontSize: 14,
          lineHeight: 22,
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          tabSize: 2,
          wordWrap: "on",
          padding: { top: 16, bottom: 20 },
        }}
      />
    </div>
  );
}
