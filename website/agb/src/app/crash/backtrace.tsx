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
      <p>If you don&apos;t want players to be sent to this page, you can:</p>
      <ol>
        <li>Configure the backtrace page to point to your own site</li>
        <li>Configure the backtrace page to not point to a site at all</li>
        <li>Not use the backtrace feature</li>
      </ol>
      <p>
        Here you can see the debug information contained in the crash log. Given
        additional debug files it can present you with the corresponding
        function names, file names, line numbers, and column numbers.
      </p>
      <Backtrace />
    </ContentBlock>
  );
}

const BacktraceListWrapper = styled.div`
  font-size: 1rem;
  position: relative;
  width: calc(100vw - 20px);
  margin-left: calc(-1 * (100vw - 20px) / 2);
  left: 50%;
`;

const BacktraceList = styled.ol`
  overflow-x: scroll;
  white-space: nowrap;
`;

function Backtrace() {
  const backtrace = useClientValue(getBacktrace) ?? "";
  const backtraceAddresses = useBacktraceData(backtrace);

  const [files, setFile] = useState<File[]>([]);
  const backtraceAddressesList =
    typeof backtraceAddresses === "object" ? backtraceAddresses : [];
  const backtraceError =
    typeof backtraceAddresses === "string" ? backtraceAddresses : undefined;

  const backtraceLocations = useBacktraceLocations(
    backtraceAddressesList,
    files ?? []
  );

  return (
    <details>
      <summary>Addresses in the backtrace</summary>
      <label>
        Elf file or GBA file with debug information:
        <input
          type="file"
          onChange={(evt) => {
            const files = evt.target.files;
            const filesArr = (files && Array.from(files)) ?? [];
            setFile(filesArr);
          }}
        />
      </label>
      <BacktraceListWrapper>
        <BacktraceList>
          {backtraceError}
          {backtraceAddressesList.map((x, idx) => (
            <li key={x}>
              {backtraceLocations[idx] ? (
                <BacktraceAddressInfo info={backtraceLocations[idx]} />
              ) : (
                <code>0x{x.toString(16).padStart(8, "0")}</code>
              )}
            </li>
          ))}
        </BacktraceList>
      </BacktraceListWrapper>
    </details>
  );
}

function makeNicePath(path: string) {
  const srcIndex = path.lastIndexOf("/src/");
  if (srcIndex < 0) return path;

  const crateNameStartIndex = path.slice(0, srcIndex).lastIndexOf("/");
  const crateName =
    crateNameStartIndex < 0
      ? "<crate>"
      : path.slice(crateNameStartIndex + 1, srcIndex);

  return `<${crateName}>/${path.slice(srcIndex + 5)}`;
}

const GreenSpan = styled.span`
  color: green;
`;

const BacktraceAddressLine = styled.ul`
  list-style-type: none;
  padding-left: 20px;
`;

function BacktraceAddressInfo({ info }: { info: AddressInfo[] | undefined }) {
  if (!info) return;

  function FunctionName({
    interesting,
    functionName,
  }: {
    interesting: boolean;
    functionName: string;
  }) {
    if (interesting) {
      return <strong>{functionName}</strong>;
    }
    return functionName;
  }

  return (
    <BacktraceAddressLine>
      {info.map((x, idx) => (
        <li key={idx}>
          <code>
            {x.is_inline && "(inlined into)"}{" "}
            <FunctionName
              interesting={x.is_interesting}
              functionName={x.function_name}
            />{" "}
            <GreenSpan>
              {makeNicePath(x.filename)}:{x.line_number}:{x.column}
            </GreenSpan>
          </code>
        </li>
      ))}
    </BacktraceAddressLine>
  );
}

function useBacktraceLocations(addresses: number[], file: File[]) {
  const debug = useAgbDebug();
  const [debugInfo, setDebugInfo] = useState<AddressInfo[][]>([]);

  useEffect(() => {
    (async () => {
      const f = file[0];
      if (!f) return;
      if (!debug) return;
      const buf = await f.arrayBuffer();
      const view = new Uint8Array(buf);

      const agbDebugFile = debug.debug_file(view);
      const debugInfo = addresses.map((x) => agbDebugFile.address_info(x));
      return debugInfo;
    })().then((x) => setDebugInfo(x ?? []));
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
    } catch (e: unknown) {
      return `${e}`;
    }
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
