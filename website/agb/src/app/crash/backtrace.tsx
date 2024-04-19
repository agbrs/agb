"use client";

import { ContentBlock } from "../contentBlock";
import { useClientValue } from "../useClientValue.hook";
import { styled } from "styled-components";
import { Debug } from "./debug";

export function BacktracePage() {
  const backtrace = useClientValue(getBacktrace);

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
      {(backtrace && <BacktraceCopyDisplay backtrace={backtrace} />) || (
        <p>No backtrace data in Url</p>
      )}
      <p>
        <em>
          The owners of this website are not necessarily the creators of the
          game you are playing.
        </em>
      </p>
      <h2>For game developers</h2>
      <p>If you don&apos;t want players to be sent to this page, you can:</p>
      <ol>
        <li>Configure the backtrace page to point to your own site</li>
        <li>Configure the backtrace page to not point to a site at all</li>
        <li>Not use the backtrace feature</li>
      </ol>
      <h2>Backtrace</h2>
      {backtrace && <Debug encodedBacktrace={backtrace} />}
    </ContentBlock>
  );
}

function BacktraceCopyDisplay({ backtrace }: { backtrace: string }) {
  return (
    <BacktraceWrapper>
      <BacktraceCodeBlock>{backtrace}</BacktraceCodeBlock>
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

const BacktraceCodeBlock = styled.code`
  font-size: 3rem;
  background-color: #dddddd;
  padding: 0px 40px;
  overflow-x: scroll;
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
  overflow-x: scroll;
`;

function getBacktrace() {
  return window.location.hash.slice(1);
}
