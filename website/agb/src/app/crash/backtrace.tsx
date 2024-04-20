"use client";

import { useState } from 'react';

import { ContentBlock } from "../contentBlock";
import { GameDeveloperSummary } from './gameDeveloperSummary';
import { styled } from "styled-components";
import { Debug } from "./debug";

export function BacktracePage() {
  const [backtrace, setBacktrace] = useState(getBacktrace);

  return (
    <ContentBlock>
      <h1>agbrs crash backtrace viewer</h1>
      <p>
        You likely got here from the link / QR code that was displayed when a
        game you were playing crashed. This is the default crash page for games
        made using the agb library.
      </p>
      <p>
        The creator of the game is <em>very</em> likely interested in the code
        below <em>along with</em> a description of what you were doing at the
        time.{" "}
        <strong>Send these to the creator of the game you are playing.</strong>
      </p>
      <BacktraceCopyDisplay backtrace={backtrace} setBacktrace={setBacktrace}/>
      <p>
        <em>
          The owners of this website are not necessarily the creators of the
          game you are playing.
        </em>
      </p>
      <h2>Backtrace</h2>
      {backtrace && <Debug encodedBacktrace={backtrace} />}
      <GameDeveloperSummary />
    </ContentBlock>
  );
}

function BacktraceCopyDisplay({ backtrace, setBacktrace }: { backtrace: string , setBacktrace: (newValue: string) => void}) {
  return (
    <BacktraceWrapper>
      <BacktraceInputBox type="text" defaultValue="Enter the backtrace code here" onChange={e => setBacktrace(e.target.value)} value={backtrace} />
      <BacktraceCopyButton
        onClick={() => {
          navigator.clipboard.writeText(backtrace);
        }}
      >
        Copy
      </BacktraceCopyButton>
    </BacktraceWrapper>
  );
}

const BacktraceInputBox = styled.input`
  font-size: larger;
  background-color: #dddddd;
  flex-grow: 999;
`;

const BacktraceWrapper = styled.section`
  display: flex;
  gap: 10px;
  justify-content: center;
  align-items: center;
  flex-wrap: wrap;
`;

const BacktraceCopyButton = styled.button`
  padding: 10px;
`;


function getBacktrace() {
  return window.location.hash.slice(1);
}
