"use client";

import { GbaKey } from "@/components/mgba/bindings";
import { Game, MgbaHandle } from "@/components/mgba/mgba";
import MgbaWrapper from "@/components/mgba/mgbaWrapper";
import { MobileController } from "@/components/mobileController/mobileController";
import { useMemo, useRef, useState } from "react";
import styled from "styled-components";

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

export function Emulator({ game }: Game) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const mgba = useRef<MgbaHandle>(null);

  const mgbaHandle = useMemo(
    () => ({
      restart: () => mgba.current?.restart(),
      buttonPress: (key: GbaKey) => mgba.current?.buttonPress(key),
      buttonRelease: (key: GbaKey) => mgba.current?.buttonRelease(key),
    }),
    []
  );

  return (
    <>
      <MgbaWrapper
        ref={mgba}
        game={game}
        onLogMessage={(category, level, message) => {
          if (category === "GBA BIOS" || category === "GBA DMA") return;
          setLogs((logs) => [...logs, { category, level, message }]);
        }}
        controlMode="focus"
      />
      <MobileController mgba={mgbaHandle} />
      <LogDisplay messages={logs} />
    </>
  );
}
