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

interface LogEntry {
  category: string;
  level: string;
  message: string;
}

const LogListEntry = styled.li`
  display: grid;
  grid-template-columns: subgrid;
  grid-column: 1 / 4;
`;

const Category = styled.span`
  color: gray;
`;

const Level = styled.span`
  color: blue;
`;

const Message = styled.span`
  color: black;
`;

function LogEntry({ category, level, message }: LogEntry) {
  return (
    <LogListEntry>
      <Category>{category}</Category>
      <Level>{level}</Level>
      <Message>{message}</Message>
    </LogListEntry>
  );
}

const LogList = styled.ol`
  max-height: 400px;
  overflow-y: scroll;
  list-style: none;
  padding: 0;
  margin: 0;
  display: grid;
  grid-template-columns: auto auto 1fr;
  column-gap: 8px;
`;

function LogDisplay({ messages }: { messages: LogEntry[] }) {
  return (
    <LogList>
      {messages.map((x, idx) => (
        <LogEntry key={idx} {...x} />
      ))}
    </LogList>
  );
}

export function Emulator({ exampleName }: { exampleName: string }) {
  const example = useMemo(() => gameUrl(exampleName), [exampleName]);
  const [logs, setLogs] = useState<LogEntry[]>([]);

  return (
    <>
      <MgbaWrapper
        gameUrl={example}
        onLogMessage={(category, level, message) => {
          if (category === "GBA BIOS") return;
          setLogs((logs) => [...logs, { category, level, message }]);
        }}
      />
      <LogDisplay messages={logs} />
    </>
  );
}
