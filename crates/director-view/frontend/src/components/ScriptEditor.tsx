import { useCallback, useRef } from 'react';
import Editor, { OnMount } from '@monaco-editor/react';
import { useProjectStore } from '@/stores/project';
import { Play, FolderOpen, AlertCircle } from 'lucide-react';

// Rhai language configuration for Monaco
const RHAI_LANGUAGE_CONFIG = {
    comments: {
        lineComment: '//',
        blockComment: ['/*', '*/'],
    },
    brackets: [
        ['{', '}'],
        ['[', ']'],
        ['(', ')'],
    ],
    autoClosingPairs: [
        { open: '{', close: '}' },
        { open: '[', close: ']' },
        { open: '(', close: ')' },
        { open: '"', close: '"' },
        { open: "'", close: "'" },
    ],
    surroundingPairs: [
        { open: '{', close: '}' },
        { open: '[', close: ']' },
        { open: '(', close: ')' },
        { open: '"', close: '"' },
        { open: "'", close: "'" },
    ],
};

// Rhai syntax highlighting (similar to Rust)
const RHAI_MONARCH_TOKENS = {
    keywords: [
        'let', 'const', 'if', 'else', 'while', 'loop', 'for', 'in',
        'break', 'continue', 'return', 'throw', 'try', 'catch',
        'fn', 'private', 'import', 'export', 'as', 'true', 'false',
        'null', 'this', 'switch', 'case', 'default',
    ],
    typeKeywords: ['int', 'float', 'string', 'bool', 'char', 'array', 'map'],
    operators: [
        '=', '>', '<', '!', '~', '?', ':', '==', '<=', '>=', '!=',
        '&&', '||', '++', '--', '+', '-', '*', '/', '&', '|', '^', '%',
        '<<', '>>', '+=', '-=', '*=', '/=', '&=', '|=', '^=',
        '%=', '<<=', '>>=',
    ],
    symbols: /[=><!~?:&|+\-*\/\^%]+/,
    escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,
    tokenizer: {
        root: [
            [/[a-z_$][\w$]*/, {
                cases: {
                    '@typeKeywords': 'keyword.type',
                    '@keywords': 'keyword',
                    '@default': 'identifier',
                },
            }],
            [/[A-Z][\w\$]*/, 'type.identifier'],
            { include: '@whitespace' },
            [/[{}()\[\]]/, '@brackets'],
            [/[<>](?!@symbols)/, '@brackets'],
            [/@symbols/, {
                cases: {
                    '@operators': 'operator',
                    '@default': '',
                },
            }],
            [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
            [/0[xX][0-9a-fA-F]+/, 'number.hex'],
            [/\d+/, 'number'],
            [/[;,.]/, 'delimiter'],
            [/"([^"\\]|\\.)*$/, 'string.invalid'],
            [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],
            [/'[^\\']'/, 'string'],
            [/(')(@escapes)(')/, ['string', 'string.escape', 'string']],
            [/'/, 'string.invalid'],
        ],
        comment: [
            [/[^\/*]+/, 'comment'],
            [/\/\*/, 'comment', '@push'],
            ['\\*/', 'comment', '@pop'],
            [/[\/*]/, 'comment'],
        ],
        string: [
            [/[^\\"]+/, 'string'],
            [/@escapes/, 'string.escape'],
            [/\\./, 'string.escape.invalid'],
            [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
        ],
        whitespace: [
            [/[ \t\r\n]+/, 'white'],
            [/\/\*/, 'comment', '@comment'],
            [/\/\/.*$/, 'comment'],
        ],
    },
};

export function ScriptEditor() {
    const {
        scriptContent,
        scriptPath,
        scriptError,
        isScriptDirty,
        isLoading,
        setScript,
        runScript,
        loadScriptFromPath,
    } = useProjectStore();

    const editorRef = useRef<Parameters<OnMount>[0] | null>(null);

    const handleEditorMount: OnMount = (editor, monaco) => {
        editorRef.current = editor;

        // Register Rhai language
        monaco.languages.register({ id: 'rhai' });
        monaco.languages.setLanguageConfiguration('rhai', RHAI_LANGUAGE_CONFIG);
        monaco.languages.setMonarchTokensProvider('rhai', RHAI_MONARCH_TOKENS as any);

        // Add keyboard shortcut for running script
        editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
            runScript();
        });

        // Add keyboard shortcut for save (placeholder)
        editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
            // TODO: Implement save to file
            console.log('Save requested');
        });
    };

    const handleChange = useCallback((value: string | undefined) => {
        setScript(value ?? '');
    }, [setScript]);

    const handleOpenFile = useCallback(() => {
        // For now, prompt for path. In Tauri, this would use native file dialog
        const path = window.prompt('Enter script path:');
        if (path) {
            loadScriptFromPath(path);
        }
    }, [loadScriptFromPath]);

    return (
        <div className="flex flex-col h-full">
            {/* Toolbar */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-director-border bg-director-surface/50">
                <div className="flex items-center gap-2">
                    <span className="text-sm text-director-text-muted">Script</span>
                    {scriptPath && (
                        <span className="text-xs text-director-text-muted truncate max-w-[200px]" title={scriptPath}>
                            {scriptPath.split(/[/\\]/).pop()}
                        </span>
                    )}
                    {isScriptDirty && (
                        <span className="text-xs text-director-accent">•</span>
                    )}
                </div>
                <div className="flex items-center gap-1">
                    <button
                        onClick={handleOpenFile}
                        className="btn-icon"
                        title="Open file"
                    >
                        <FolderOpen size={16} />
                    </button>
                    <button
                        onClick={() => runScript()}
                        disabled={isLoading}
                        className="btn-icon text-director-accent hover:bg-director-accent hover:text-white"
                        title="Run script (Ctrl+Enter)"
                    >
                        <Play size={16} />
                    </button>
                </div>
            </div>

            {/* Error display */}
            {scriptError && (
                <div className="px-3 py-2 bg-red-500/10 border-b border-red-500/30 flex items-start gap-2">
                    <AlertCircle size={16} className="text-red-400 mt-0.5 flex-shrink-0" />
                    <pre className="text-xs text-red-400 whitespace-pre-wrap font-mono overflow-auto max-h-32">
                        {scriptError}
                    </pre>
                </div>
            )}

            {/* Editor */}
            <div className="flex-1 overflow-hidden">
                <Editor
                    height="100%"
                    language="rhai"
                    theme="vs-dark"
                    value={scriptContent}
                    onChange={handleChange}
                    onMount={handleEditorMount}
                    options={{
                        minimap: { enabled: false },
                        fontSize: 13,
                        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
                        lineNumbers: 'on',
                        scrollBeyondLastLine: false,
                        automaticLayout: true,
                        tabSize: 2,
                        wordWrap: 'on',
                        padding: { top: 8, bottom: 8 },
                        renderLineHighlight: 'line',
                        cursorBlinking: 'smooth',
                        smoothScrolling: true,
                    }}
                />
            </div>

            {/* Status bar */}
            <div className="flex items-center justify-between px-3 py-1 border-t border-director-border bg-director-surface/50 text-xs text-director-text-muted">
                <span>Rhai Script</span>
                <span>Ctrl+Enter to run</span>
            </div>
        </div>
    );
}
