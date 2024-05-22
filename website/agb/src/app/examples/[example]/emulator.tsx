"use client";

import MgbaWrapper from "@/components/mgba/mgbaWrapper";
import { Examples } from "@/roms/examples/examples";
import { slugify } from "@/sluggify";
import { useMemo } from "react";

function gameUrl(exampleName: string) {
  const example = Examples.find((x) => slugify(x.example_name) === exampleName);
  console.log(exampleName);
  if (!example) {
    throw new Error(`cannot find example ${exampleName}`);
  }

  return example.url;
}

export function Emulator({ exampleName }: { exampleName: string }) {
  const example = useMemo(() => gameUrl(exampleName), [exampleName]);

  return <MgbaWrapper gameUrl={example} />;
}
