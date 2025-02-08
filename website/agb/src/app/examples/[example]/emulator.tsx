"use client";

import MgbaWrapper from "@/components/mgba/mgbaWrapper";
import { Examples } from "@/roms/examples/examples";
import { slugify } from "@/sluggify";
import { useMemo, useState } from "react";
import styled from "styled-components";

function gameUrl(exampleName: string) {
  const example = Examples.find((x) => slugify(x.example_name) === exampleName);
  if (!example) {
    throw new Error(`cannot find example ${exampleName}`);
  }

  return example.url;
}

const LogList = styled.ol`
  max-height: 400px;
  overflow-y: scroll;
`;

function LogDisplay({ messages }: { messages: string[] }) {
  return (
    <LogList>
      {messages.map((x, idx) => (
        <li key={idx}>{x}</li>
      ))}
    </LogList>
  );
}

export function Emulator({ exampleName }: { exampleName: string }) {
  const example = useMemo(() => gameUrl(exampleName), [exampleName]);
  const [logs, setLogs] = useState<string[]>([]);

  return (
    <>
      <MgbaWrapper
        gameUrl={example}
        onLogMessage={(category, level, message) => {
          if (category === "GBA BIOS") return;
          setLogs((logs) => [...logs, `${category} ${level} ${message}`]);
        }}
      />
      <LogDisplay messages={logs} />
    </>
  );
}
