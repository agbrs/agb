import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import mGBA, { mGBAEmulator, LogLevel } from "./vendor/mgba";
import { GbaKey, KeyBindings } from "./bindings";
import { styled } from "styled-components";
import { useController } from "./useController.hook";
import { useLocalStorage } from "./useLocalStorage.hook";

interface MgbaProps {
  gameUrl: URL;
  volume?: number;
  controls: KeyBindings;
  paused: boolean;
  onLogMessage?: (category: string, level: LogLevel, message: string) => void;
}

enum MgbaState {
  Uninitialised,
  Initialising,
  Initialised,
}

const MGBA_ROM_DIRECTORY = "/data/games";

const MgbaCanvas = styled.canvas`
  image-rendering: pixelated;
  aspect-ratio: 240 / 160;
  width: 100%;
  object-fit: contain;
  max-height: 100%;
`;

export interface MgbaHandle {
  restart: () => void;
  buttonPress: (key: GbaKey) => void;
  buttonRelease: (key: GbaKey) => void;
}

async function downloadGame(gameUrl: URL): Promise<ArrayBuffer> {
  const game = await fetch(gameUrl);

  const gameUrlString = gameUrl.toString();

  if (gameUrlString.endsWith(".gz")) {
    const decompressedStream = (await game.blob())
      .stream()
      .pipeThrough(new DecompressionStream("gzip"));
    return await new Response(decompressedStream).arrayBuffer();
  } else {
    return await game.arrayBuffer();
  }
}

interface SaveGame {
  [gameName: string]: number[];
}

export const Mgba = forwardRef<MgbaHandle, MgbaProps>(
  ({ gameUrl, volume, controls, paused, onLogMessage }, ref) => {
    const canvas = useRef(null);
    const mgbaModule = useRef<mGBAEmulator>(undefined);

    const [saveGame, setSaveGame] = useLocalStorage<SaveGame>(
      {},
      "agbrswebplayer/savegames"
    );
    const gameUrlString = gameUrl.toString();

    const [state, setState] = useState(MgbaState.Uninitialised);
    const [gameLoaded, setGameLoaded] = useState(false);

    useEffect(() => {
      if (state !== MgbaState.Initialised) return;

      function logListener(category: string, level: LogLevel, message: string) {
        if (onLogMessage) onLogMessage(category, level, message);
      }
      mgbaModule.current?.addLogListener(logListener);

      return () => {
        mgbaModule.current?.removeLogListener(logListener);
      };
    }, [onLogMessage, state]);

    useEffect(() => {
      function beforeUnload() {
        const gameSplit = gameUrlString.split("/");
        const gameBaseName = gameSplit[gameSplit.length - 1];

        const save = mgbaModule.current?.getSave();
        if (!save) return;

        setSaveGame({
          ...saveGame,
          [gameBaseName]: [...save],
        });
      }

      window.addEventListener("beforeunload", beforeUnload);

      return () => {
        window.removeEventListener("beforeunload", beforeUnload);
      };
    }, [gameUrlString, saveGame, setSaveGame]);

    useEffect(() => {
      if (state !== MgbaState.Initialised) return;

      const gameSplit = gameUrlString.split("/");
      const gameBaseName = gameSplit[gameSplit.length - 1];

      const save = saveGame[gameBaseName];
      if (!save) return;

      const savePath = `${MGBA_ROM_DIRECTORY}/${gameBaseName}.sav`;

      mgbaModule.current?.FS.writeFile(savePath, new Uint8Array([0, 1, 2, 3]));
    }, [gameUrlString, saveGame, state]);

    useEffect(() => {
      if (state !== MgbaState.Initialised) return;
      (async () => {
        const gameData = await downloadGame(gameUrl);
        const gameSplit = gameUrlString.split("/");
        const gameBaseName = gameSplit[gameSplit.length - 1];

        const gamePath = `${MGBA_ROM_DIRECTORY}/${gameBaseName}`;
        mgbaModule.current?.FS.writeFile(gamePath, new Uint8Array(gameData));
        mgbaModule.current?.loadGame(gamePath);
        mgbaModule.current?.setVolume(0.1); // for some reason you have to do this or you get no sound
        setGameLoaded(true);
      })();
    }, [state, gameUrl, gameUrlString]);

    // init mgba
    useEffect(() => {
      (async () => {
        if (canvas.current === null) return;
        if (state !== MgbaState.Uninitialised) return;

        setState(MgbaState.Initialising);

        const mModule = await mGBA({ canvas: canvas.current });
        mgbaModule.current = mModule;
        await mModule.FSInit();
        await mModule.FSSync();
        setState(MgbaState.Initialised);
      })();

      if (state === MgbaState.Initialised)
        return () => {
          try {
            mgbaModule.current?.quitGame();
            mgbaModule.current?.quitMgba();
          } catch {}
        };
    }, [state]);

    useController(mgbaModule);

    useEffect(() => {
      if (!gameLoaded) return;

      const controlEntries = Object.entries(controls);

      for (const [key, value] of controlEntries) {
        const binding =
          value === "Enter"
            ? "Return"
            : value.toLowerCase().replace("arrow", "").replace("key", "");

        mgbaModule.current?.bindKey(binding, key);
      }
    }, [controls, gameLoaded]);

    useEffect(() => {
      if (!gameLoaded) return;
      mgbaModule.current?.setVolume(volume ?? 1.0);
    }, [gameLoaded, volume]);

    useEffect(() => {
      if (!gameLoaded) return;

      if (paused) {
        mgbaModule.current?.pauseGame();
      } else {
        mgbaModule.current?.resumeGame();
      }
    }, [gameLoaded, paused]);

    useImperativeHandle(ref, () => {
      return {
        restart: () => mgbaModule.current?.quickReload(),
        buttonPress: (key: GbaKey) => mgbaModule.current?.buttonPress(key),
        buttonRelease: (key: GbaKey) => mgbaModule.current?.buttonUnpress(key),
        saveGame: () => {},
      };
    });

    return <MgbaCanvas ref={canvas} />;
  }
);
Mgba.displayName = "Mgba";
