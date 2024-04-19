import { useEffect, useState } from "react";
import debugInit, {
  decode_backtrace,
  DebugFile,
  InitOutput,
  AddressInfo,
} from "./vendor/backtrace/backtrace";

let agbDebug: Promise<InitOutput> | undefined;

export { AddressInfo };

export interface AgbDebug {
  decode_backtrace: (backtrace: string) => Uint32Array;
  debug_file: (file: Uint8Array) => DebugFile;
}

export function useAgbDebug(): AgbDebug | undefined {
  const [debug, setDebug] = useState<AgbDebug>();

  useEffect(() => {
    (async () => {
      if (agbDebug === undefined) {
        agbDebug = debugInit();
      }

      await agbDebug;

      setDebug({
        decode_backtrace,
        debug_file: (file: Uint8Array) => new DebugFile(file),
      });
    })();
  }, []);

  return debug;
}
