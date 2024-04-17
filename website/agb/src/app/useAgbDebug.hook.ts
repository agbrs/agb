import { useEffect, useState } from "react";
import debugInit, {
  decode_backtrace,
  DebugFile,
  InitOutput,
} from "./vendor/agb_wasm/agb_wasm";

let agbDebug: Promise<InitOutput> | undefined;

interface AgbDebug {
  decode_backtrace: (backtrace: string) => Uint32Array;
  debug_file: (file: Uint8Array) => DebugFile;
}

export function useAgbDebug() {
  const [debug, setDebug] = useState<AgbDebug>();

  useEffect(() => {
    (async () => {
      if (agbDebug === undefined) {
        agbDebug = debugInit();
      }

      await agbDebug;
      console.log("Loaded agb debug");

      setDebug({
        decode_backtrace,
        debug_file: (file: Uint8Array) => new DebugFile(file),
      });
    })();
  }, []);

  return debug;
}
