"use client";

import { Emulator } from "@/app/examples/[example]/emulator";
import { Editor, EditorRef } from "@/components/editor/editor";
import {
  ReactNode,
  Ref,
  useCallback,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
  useTransition,
} from "react";
import { styled } from "styled-components";
import { Game } from "@/components/mgba/mgba";
import { Flex } from "@/components/flex";
import { Resizable } from "@/components/resizable";
import { keymap } from "@codemirror/view";
import { Ansi } from "@/components/ansi";

const FullHeightEditor = styled(Editor)`
  height: 100%;
`;

const Spinner = styled.span`
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  display: inline-block;
  width: 1em;
  height: 1em;
  border: 2px solid rgba(0, 0, 0, 0.2);
  border-top-color: black;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  vertical-align: middle;
`;

export const ActionButton = styled.button`
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 5px 10px;
  font-size: 1rem;
  text-decoration: none;
  color: black;
  background-color: #fad288;
  border: solid #fad288 2px;
  border-radius: 5px;
  cursor: pointer;

  &:hover:not(:disabled) {
    border-color: black;
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;

const Container = styled(Resizable)`
  height: 100%;
  min-height: 0;
  padding: 8px;
`;

const ErrorDisplay = styled.div`
  overflow-y: scroll;
  width: 100%;
  font-size: 12px;
`;

const ButtonRow = styled.div`
  display: flex;
  gap: 8px;
`;

const BUILD_URL = process.env.NEXT_PUBLIC_PLAYGROUND_URL;

export interface CodeRunnerHandle {
  build: () => void;
  getCode: () => string;
}

export interface CodeRunnerProps {
  sourceCode: string;
  initialGame?: Game["game"] | null;
  extraButtons?: (props: { isPending: boolean }) => ReactNode;
  ref?: Ref<CodeRunnerHandle>;
}

export function CodeRunner({
  sourceCode,
  initialGame = null,
  extraButtons,
  ref,
}: CodeRunnerProps) {
  const [game, setGame] = useState<Game["game"] | null>(initialGame);
  const [error, setError] = useState("");
  const codeRef = useRef<EditorRef>(null);

  const [isPending, startTransition] = useTransition();

  const buildTransition = useCallback(() => {
    async function build() {
      if (!codeRef.current) return;

      const code = codeRef.current.toString();

      const response = await fetch(`${BUILD_URL}/build`, {
        method: "post",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ code }),
      });

      if (response.status !== 200) {
        const json = await response.json();
        startTransition(() => {
          setError(json["error"] ?? "");
        });
      } else {
        const decompressedStream = (await response.blob())
          .stream()
          .pipeThrough(new DecompressionStream("gzip"));
        const game = await new Response(decompressedStream).arrayBuffer();

        startTransition(() => {
          setError("");
          setGame(game);
        });
      }
    }

    startTransition(async () => {
      try {
        await build();
      } catch (e) {
        setError(`Could not build due to unknown failure: ${e}`);
      }
    });
  }, []);

  useImperativeHandle(ref, () => ({
    build: buildTransition,
    getCode: () => codeRef.current?.toString() ?? "",
  }));

  const buildExtension = useMemo(
    () => [
      keymap.of([
        {
          key: "Ctrl-Enter",
          run: () => {
            buildTransition();
            return true;
          },
        },
      ]),
    ],
    [buildTransition]
  );

  return (
    <Container
      left={
        <FullHeightEditor
          defaultContent={sourceCode}
          ref={codeRef}
          extensions={buildExtension}
        />
      }
      right={
        <Flex $v>
          <ButtonRow>
            <ActionButton disabled={isPending} onClick={buildTransition}>
              {isPending && <Spinner />}
              {isPending ? "Building..." : "Build and Run"}
            </ActionButton>
            {extraButtons?.({ isPending })}
          </ButtonRow>
          {!!error || (game && <Emulator game={game} />)}
          {error && (
            <ErrorDisplay>
              <Ansi text={error} />
            </ErrorDisplay>
          )}
        </Flex>
      }
    />
  );
}
