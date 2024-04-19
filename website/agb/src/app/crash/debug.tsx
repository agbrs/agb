import { styled } from "styled-components";
import { AddressInfo, AgbDebug, useAgbDebug } from "../useAgbDebug.hook";
import { useMemo, useState } from "react";

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

interface DebugProps {
  encodedBacktrace: string;
}

export function Debug(props: DebugProps) {
  const debug = useAgbDebug();
  if (debug) {
    return <DebugBacktraceDecode debug={debug} {...props} />;
  } else {
    return <p>Loading debug viewer...</p>;
  }
}

interface DebugBacktraceDecodeProps extends DebugProps {
  debug: AgbDebug;
}

const NonWrapCode = styled.code`
  white-space: nowrap;
`;

function DebugBacktraceDecode({
  encodedBacktrace,
  debug,
}: DebugBacktraceDecodeProps) {
  const backtraceAddresses = useBacktraceData(debug, encodedBacktrace);
  const [backtraceLocations, setBacktraceLocations] = useState<AddressInfo[][]>(
    []
  );

  if (typeof backtraceAddresses === "string") {
    return <DebugError error={backtraceAddresses} />;
  }

  return (
    <>
      <p>
        If you add the elf file used to make the GBA file, or the GBA file
        itself if it was made with <NonWrapCode>agb-gbafix --debug</NonWrapCode>
        , you can see: function names, file names, line numbers, and column
        numbers.
      </p>
      <label>
        Elf file or GBA file with debug information:{" "}
        <input
          type="file"
          onChange={(evt) => {
            const files = evt.target.files;
            if (!files) return;
            const file = files[0];
            if (!file) return;
            loadLocations(debug, backtraceAddresses, file).then((data) =>
              setBacktraceLocations(data)
            );
          }}
        />
      </label>
      <BacktraceListWrapper>
        <BacktraceList>
          {backtraceAddresses.map((x, idx) => (
            <li key={x}>
              <BacktraceAddressInfo
                address={x}
                info={backtraceLocations[idx]}
              />
            </li>
          ))}
        </BacktraceList>
      </BacktraceListWrapper>
    </>
  );
}

function DebugError({ error }: { error: string }) {
  return (
    <>
      <p>Something went wrong decoding the backtrace</p>
      <p>{error}</p>
    </>
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

function BacktraceAddressInfo({
  address,
  info,
}: {
  address: number;
  info: AddressInfo[] | undefined;
}) {
  if (!info) {
    return <code>0x{address.toString(16).padStart(8, "0")}</code>;
  }

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

async function loadLocations(debug: AgbDebug, addresses: number[], file: File) {
  const buf = await file.arrayBuffer();
  const view = new Uint8Array(buf);

  const agbDebugFile = debug.debug_file(view);
  const debugInfo = addresses.map((x) => agbDebugFile.address_info(x));
  return debugInfo;
}

function useBacktraceData(debug: AgbDebug, trace: string) {
  return useMemo(() => {
    try {
      const addresses = debug?.decode_backtrace(trace);
      return addresses && Array.from(addresses);
    } catch (e: unknown) {
      return `${e}`;
    }
  }, [debug, trace]);
}
