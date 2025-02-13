"use client";

import { Emulator } from "./emulator";
import { Editor, EditorRef } from "@/components/editor/editor";
import { useCallback, useMemo, useRef, useState, useTransition } from "react";
import { slugify } from "@/sluggify";
import { Examples } from "@/roms/examples/examples";
import { styled } from "styled-components";
import { Game } from "@/components/mgba/mgba";
import { Flex } from "@/components/flex";
import { Resizable } from "@/components/resizable";
import { keymap } from "@codemirror/view";
import { Ansi } from "@/components/ansi";

export interface ExampleProps {
  exampleSlug: string;
  sourceCode: string;
}

function gameUrl(exampleName: string) {
  const example = Examples.find((x) => slugify(x.example_name) === exampleName);
  if (!example) {
    throw new Error(`cannot find example ${exampleName}`);
  }

  return example.url;
}

const FullHeightEditor = styled(Editor)`
  height: 100%;
`;

const RunButton = styled.button``;

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

export function Example({ exampleSlug, sourceCode }: ExampleProps) {
  const [game, setGame] = useState<Game["game"]>(() => gameUrl(exampleSlug));
  const [error, setError] = useState("");
  const codeRef = useRef<EditorRef>(null);

  const [isPending, startTransition] = useTransition();

  const buildTransition = useCallback(() => {
    async function build() {
      if (!codeRef.current) return;

      const code = codeRef.current.toString();

      const response = await fetch("http://localhost:5409/build", {
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
          <RunButton disabled={isPending} onClick={buildTransition}>
            Build and Run
          </RunButton>
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
