import { styled } from "styled-components";
import { AddressInfo, AgbDebug, useAgbDebug } from "../useAgbDebug.hook";
import { ReactNode, useMemo, useState } from "react";

const BacktraceListWrapper = styled.div`
  font-size: 1rem;
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

  const [backtraceLocationsError, setBacktraceLocationsError] =
    useState<string>("");

  if (typeof backtraceAddresses === "string") {
    return (
      <DebugError>
        Something went wrong decoding the backtrace: {backtraceAddresses}
      </DebugError>
    );
  }

  return (
    <>
      <BacktraceListWrapper>
        <BacktraceList>
          {backtraceAddresses.map((x, idx) => (
            <li key={idx}>
              <BacktraceAddressInfo
                address={x}
                info={backtraceLocations[idx]}
              />
            </li>
          ))}
        </BacktraceList>
      </BacktraceListWrapper>
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
            setBacktraceLocationsError("");
            loadLocations(debug, backtraceAddresses, file)
              .then((data) => setBacktraceLocations(data))
              .catch((e) => setBacktraceLocationsError(`${e}`));
          }}
        />
      </label>
      {backtraceLocationsError && (
        <DebugError>
          Something went wrong looking up the addresses in the file provided:{" "}
          {backtraceLocationsError}
        </DebugError>
      )}
    </>
  );
}

const ErrorBlock = styled.div`
  background-color: #f78f8f;
  border: 2px solid #9c0a0a;
  border-radius: 8px;
  padding: 20px;
  margin-top: 10px;
`;

function DebugError({ children }: { children: ReactNode }) {
  return <ErrorBlock>{children}</ErrorBlock>;
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
  const formattedAddress = `0x${address.toString(16).padStart(8, "0")}`;
  if (!info) {
    return <code>{formattedAddress}</code>;
  }

  if (info.length === 0) {
    return (
      <BacktraceAddressLine>
        <li>
          <code>(no info) {formattedAddress}</code>
        </li>
      </BacktraceAddressLine>
    );
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

  console.log(info);

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
