"use client";

import {
  Ref,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
  useTransition,
} from "react";
import mGBA, { mGBAEmulator, LogLevel } from "./vendor/mgba";
import { GbaKey, KeyBindings } from "./bindings";
import { styled } from "styled-components";
import { useController } from "./useController.hook";
import { useLocalStorage } from "./useLocalStorage.hook";

export interface Game {
  game: URL | string | ArrayBuffer;
}

async function generateGameName({ game }: Game): Promise<string> {
  if (typeof game === "string") {
    const split = game.split("/");
    return split[split.length - 2];
  }

  if (game instanceof URL) {
    const split = game.toString().split("/");
    return split[split.length - 2];
  }

  const crypto = window.crypto.subtle;
  const buffer = await crypto.digest("SHA-1", game);
  const hashArray = Array.from(new Uint8Array(buffer));
  const hashHex = hashArray
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  return hashHex;
}

interface MgbaProps extends Game {
  volume?: number;
  controls: KeyBindings;
  paused: boolean;
  onLogMessage?: (category: string, level: LogLevel, message: string) => void;
  ref?: Ref<MgbaHandle> | undefined;
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

async function downloadGame({ game }: Game): Promise<ArrayBuffer> {
  if (game instanceof ArrayBuffer) return game;

  const data = await fetch(game);

  const gameUrlString = game.toString();

  if (gameUrlString.endsWith(".gz")) {
    const decompressedStream = (await data.blob())
      .stream()
      .pipeThrough(new DecompressionStream("gzip"));
    return await new Response(decompressedStream).arrayBuffer();
  } else {
    return await data.arrayBuffer();
  }
}

interface SaveGame {
  [gameName: string]: number[];
}

interface MgbaInnerProps {
  game: ArrayBuffer;
  gameName: string;
  volume?: number;
  controls: KeyBindings;
  paused: boolean;
  onLogMessage?: (category: string, level: LogLevel, message: string) => void;
  ref?: Ref<MgbaHandle> | undefined;
}

function MgbaInner({
  game,
  gameName,
  volume,
  controls,
  paused,
  onLogMessage,
  ref,
}: MgbaInnerProps) {
  const canvas = useRef(null);
  const mgbaModule = useRef<mGBAEmulator>(undefined);

  const [saveGame, setSaveGame] = useLocalStorage<SaveGame>(
    {},
    "agbrswebplayer/savegames"
  );

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
      const save = mgbaModule.current?.getSave();
      if (!save) return;

      setSaveGame({
        ...saveGame,
        [gameName]: [...save],
      });
    }

    window.addEventListener("beforeunload", beforeUnload);

    return () => {
      window.removeEventListener("beforeunload", beforeUnload);
    };
  }, [gameName, saveGame, setSaveGame]);

  useEffect(() => {
    if (state !== MgbaState.Initialised) return;

    const save = saveGame[gameName];
    if (!save) return;

    const savePath = `${MGBA_ROM_DIRECTORY}/${gameName}.sav`;

    mgbaModule.current?.FS.writeFile(savePath, new Uint8Array([0, 1, 2, 3]));
  }, [gameName, saveGame, state]);

  useEffect(() => {
    if (state !== MgbaState.Initialised) return;
    (async () => {
      const gamePath = `${MGBA_ROM_DIRECTORY}/${gameName}`;
      mgbaModule.current?.FS.writeFile(gamePath, new Uint8Array(game));
      mgbaModule.current?.loadGame(gamePath);
      mgbaModule.current?.setVolume(0.1); // for some reason you have to do this or you get no sound
      setGameLoaded(true);
    })();
  }, [state, gameName, game]);

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

export function Mgba({
  game,
  volume,
  controls,
  paused,
  onLogMessage,
  ref,
}: MgbaProps) {
  const [isPending, startTransition] = useTransition();
  const [gameData, setGameData] = useState<{
    game: ArrayBuffer;
    gameName: string;
  } | null>(null);

  useEffect(() => {
    (async () => {
      startTransition(async () => {
        try {
          const [gameData, gameName] = await Promise.all([
            downloadGame({ game }),
            generateGameName({ game }),
          ]);
          startTransition(() => setGameData({ game: gameData, gameName }));
        } catch {
          startTransition(() => setGameData(null));
        }
      });
    })();
  }, [game]);

  if (isPending) return <>Loading...</>;
  if (!gameData) return <>Failed to load game</>;

  return (
    <MgbaInner
      {...gameData}
      volume={volume}
      controls={controls}
      paused={paused}
      onLogMessage={onLogMessage}
      ref={ref}
    />
  );
}
