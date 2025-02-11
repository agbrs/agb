"use client";

import { ContentBlock } from "@/components/contentBlock";
import { Emulator } from "./emulator";
import { Editor, EditorRef } from "@/components/editor/editor";
import { useRef, useState, useTransition } from "react";
import { slugify } from "@/sluggify";
import { Examples } from "@/roms/examples/examples";
import { styled } from "styled-components";
import { Game } from "@/components/mgba/mgba";

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

const RunButton = styled.button``;

export function Example({ exampleSlug, sourceCode }: ExampleProps) {
  const [game, setGame] = useState<Game["game"]>(() => gameUrl(exampleSlug));
  const codeRef = useRef<EditorRef>(null);

  const [isPending, startTransition] = useTransition();

  async function buildAndRun() {
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

    const decompressedStream = (await response.blob())
      .stream()
      .pipeThrough(new DecompressionStream("gzip"));
    const game = await new Response(decompressedStream).arrayBuffer();

    return game;
  }

  return (
    <>
      {game && (
        <ContentBlock>
          <Emulator game={game} />
        </ContentBlock>
      )}

      <ContentBlock>
        <RunButton
          disabled={isPending}
          onClick={() => {
            startTransition(async () => {
              try {
                const game = await buildAndRun();
                startTransition(() => {
                  if (game) setGame(game);
                });
              } catch {}
            });
          }}
        >
          Build and Run
        </RunButton>
        <Editor defaultContent={sourceCode} ref={codeRef} />
      </ContentBlock>
    </>
  );
}
