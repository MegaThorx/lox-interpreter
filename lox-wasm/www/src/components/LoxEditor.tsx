import Editor from "@monaco-editor/react";
import { useRef } from "react";
import { registerLox } from "../loxLanguageConfig";

function LoxEditor({value, onChange}: {value: string, onChange: (value: string | undefined) => void}) {
  const editorRef = useRef(null);

  const handleEditorWillMount = (monaco: any) => {
    registerLox(monaco);
  };

  const handleEditorDidMount = (editor: any, _monaco: any) => {
    editorRef.current = editor;
  };

  return (
    <Editor
      height="100%"
      language="lox"
      theme="vs-dark"
      value={value}
      onChange={onChange}
      beforeMount={handleEditorWillMount}
      onMount={handleEditorDidMount}
      options={{
        minimap: { enabled: false },
        fontSize: 14,
        scrollBeyondLastLine: false,
        automaticLayout: true,
      }}
    />
  );
};

export default LoxEditor;