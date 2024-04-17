"use client";

import { ContentBlock } from "../contentBlock";
import { useClientValue } from "../useClientValue.hook";
import { styled } from "styled-components";
import { useEffect, useMemo, useState } from "react";
import { useAgbDebug } from "../useAgbDebug.hook";
import { AddressInfo } from "../vendor/agb_wasm/agb_wasm";

export function BacktracePage() {
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
      <BacktraceDisplay />
      <p>
        <em>
          The owners of this website are not necessarily the creators of the
          game you are playing.
        </em>
      </p>
      <h2>For game developers</h2>
      <p>This page will eventually let you view backtraces in the browser.</p>
      <p>
        For now you can copy the backtrace code here and use it with{" "}
        <code>agb-addr2line</code>.
      </p>
      <p>If you don&apos;t want players to be sent to this page, you can:</p>
      <ol>
        <li>Configure the backtrace page to point to your own site</li>
        <li>Configure the backtrace page to not point to a site at all</li>
        <li>Not use the backtrace feature</li>
      </ol>
      <Backtrace />
    </ContentBlock>
  );
}

function Backtrace() {
  const backtrace = useClientValue(getBacktrace) ?? "";
  const backtraceAddresses = useBacktraceData(backtrace);

  const [files, setFile] = useState<File[]>([]);
  const backtraceLocations = useBacktraceLocations(
    backtraceAddresses ?? [],
    files ?? []
  );

  return (
    <details>
      <summary>Addresses in the backtrace</summary>
      <input
        type="file"
        onChange={(evt) => {
          const files = evt.target.files;
          const filesArr = (files && Array.from(files)) ?? [];
          setFile(filesArr);
        }}
      />
      <ol>
        {backtraceAddresses &&
          backtraceAddresses.map((x, idx) => (
            <li key={x}>
              <code>0x{x.toString(16).padStart(8, "0")}</code>
              <BacktraceAddressInfo info={backtraceLocations[idx]} />
            </li>
          ))}
      </ol>
    </details>
  );
}

function BacktraceAddressInfo({ info }: { info: AddressInfo[] | undefined }) {
  if (!info) return;

  return (
    <ol>
      {info.map((x, idx) => (
        <li key={idx}>
          {x.is_inline && "(inlined into)"} {x.function_name}:{x.column}{" "}
          {x.filename}:{x.line_number}
        </li>
      ))}
    </ol>
  );
}

function useBacktraceLocations(addresses: number[], file: File[]) {
  const debug = useAgbDebug();
  const [debugInfo, setDebugInfo] = useState<AddressInfo[][]>([]);

  useEffect(() => {
    const f = file[0];
    if (!f) return;
    if (!debug) return;
    (async () => {
      const buf = await f.arrayBuffer();
      const view = new Uint8Array(buf);

      const agbDebugFile = debug.debug_file(view);
      const debugInfo = addresses.map((x) => agbDebugFile.address_info(x));
      setDebugInfo(debugInfo);
    })();
  }, [addresses, debug, file]);

  return debugInfo;
}

function useBacktraceData(trace?: string) {
  const debug = useAgbDebug();

  return useMemo(() => {
    try {
      if (!trace) return;
      const addresses = debug?.decode_backtrace(trace);
      return addresses && Array.from(addresses);
    } catch {}
  }, [debug, trace]);
}

function BacktraceDisplay() {
  const backtrace = useClientValue(getBacktrace) ?? "";

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
