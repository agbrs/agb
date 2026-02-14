"use client";

import { slugify } from "@/sluggify";
import { Examples } from "@/roms/examples/examples";
import { CodeRunner } from "@/components/codeRunner";

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

export function Example({ exampleSlug, sourceCode }: ExampleProps) {
  return (
    <CodeRunner
      sourceCode={sourceCode}
      initialGame={gameUrl(exampleSlug)}
    />
  );
}
