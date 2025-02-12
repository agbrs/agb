"use client";

import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
} from "@codemirror/autocomplete";
import {
  history,
  defaultKeymap,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import { rust } from "@codemirror/lang-rust";
import {
  bracketMatching,
  defaultHighlightStyle,
  foldGutter,
  foldKeymap,
  indentOnInput,
  syntaxHighlighting,
  indentUnit,
} from "@codemirror/language";
import { lintKeymap } from "@codemirror/lint";
import { highlightSelectionMatches, searchKeymap } from "@codemirror/search";
import { EditorState, StateEffect, Text } from "@codemirror/state";
import {
  crosshairCursor,
  drawSelection,
  dropCursor,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  keymap,
  lineNumbers,
  rectangularSelection,
  ViewUpdate,
} from "@codemirror/view";
import {
  ReactNode,
  Ref,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from "react";

export class EditorText {
  private text: Text;
  constructor(text: Text) {
    this.text = text;
  }

  public toString() {
    return this.text.toString();
  }
}

export interface EditorRef {
  toString: () => string;
}

interface EditorProps {
  defaultContent?: string;
  onChange?: (text: EditorText) => void;
  ref?: Ref<EditorRef> | undefined;
  className?: string;
}

const theme = EditorView.theme({
  "&": {
    fontSize: "12px",
    minHeight: "100%",
    height: "100%",
  },
  "& > .cm-scroller": {
    minHeight: "100%",
    height: "100%",
  },
});

function defaultExtensions() {
  return [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    history(),
    foldGutter(),
    drawSelection(),
    dropCursor(),
    EditorState.allowMultipleSelections.of(true),
    indentOnInput(),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    bracketMatching(),
    closeBrackets(),
    autocompletion(),
    rectangularSelection(),
    crosshairCursor(),
    highlightActiveLine(),
    highlightSelectionMatches(),
    keymap.of([
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...completionKeymap,
      ...lintKeymap,
      indentWithTab,
    ]),
    rust(),
    indentUnit.of("    "),
    theme,
  ];
}

export function Editor({
  defaultContent = "",
  onChange,
  ref,
  className,
}: EditorProps): ReactNode {
  const element = useRef<HTMLDivElement>(null);
  const [view, setView] = useState<EditorView | null>(null);

  const extensions = useMemo(() => {
    const updateListener = EditorView.updateListener.of((vu: ViewUpdate) => {
      if (vu.docChanged && onChange) {
        onChange(new EditorText(vu.state.doc));
      }
    });
    return [...defaultExtensions(), updateListener];
  }, [onChange]);

  useEffect(() => {
    if (!element.current) return;
    if (view !== null) return;

    const editorView = new EditorView({
      doc: defaultContent,
      extensions,
      parent: element.current,
    });

    setView(editorView);

    return () => {
      editorView.destroy();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!view) return;
    view.dispatch({ effects: StateEffect.reconfigure.of(extensions) });
  }, [extensions, view]);

  useImperativeHandle(ref, () => ({
    toString: () => view?.state.doc.toString() ?? "",
  }));

  return <div className={className} ref={element}></div>;
}
