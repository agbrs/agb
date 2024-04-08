import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import mGBA, { mGBAEmulator } from "./vendor/mgba";
import { GbaKey, KeyBindings } from "./bindings";
import { styled } from "styled-components";
import { useFrameSkip } from "./useFrameSkip.hook";

type Module = any;

interface MgbaProps {
  gameUrl: string;
  volume?: Number;
  controls: KeyBindings;
  paused: boolean;
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

const downloadGame = async (gameUrl: string): Promise<ArrayBuffer> => {
  const game = await fetch(gameUrl);

  if (gameUrl.endsWith(".gz")) {
    const decompressedStream = (await game.blob())
      .stream()
      .pipeThrough(new DecompressionStream("gzip"));
    return await new Response(decompressedStream).arrayBuffer();
  } else {
    return await game.arrayBuffer();
  }
};

export const Mgba = forwardRef<MgbaHandle, MgbaProps>(
  ({ gameUrl, volume, controls, paused }, ref) => {
    const canvas = useRef(null);
    const mgbaModule = useRef<Module>({} as mGBAEmulator);

    const [state, setState] = useState(MgbaState.Uninitialised);
    const [gameLoaded, setGameLoaded] = useState(false);

    useEffect(() => {
      if (state !== MgbaState.Initialised) return;
      (async () => {
        const gameData = await downloadGame(gameUrl);
        const gameSplit = gameUrl.split("/");
        const gameBaseName = gameSplit[gameSplit.length - 1];

        const gamePath = `${MGBA_ROM_DIRECTORY}/${gameBaseName}`;
        mgbaModule.current.FS.writeFile(gamePath, new Uint8Array(gameData));
        mgbaModule.current.loadGame(gamePath);
        mgbaModule.current.setVolume(0.1); // for some reason you have to do this or you get no sound
        setGameLoaded(true);
      })();
    }, [state, gameUrl]);

    // init mgba
    useEffect(() => {
      (async () => {
        if (canvas.current === null) return;
        if (state !== MgbaState.Uninitialised) return;

        setState(MgbaState.Initialising);
        mgbaModule.current = {
          canvas: canvas.current,
        };

        mGBA(mgbaModule.current).then((module: Module) => {
          mgbaModule.current = module;
          module.FSInit();
          setState(MgbaState.Initialised);
        });
      })();

      if (state === MgbaState.Initialised)
        return () => {
          try {
            mgbaModule.current.quitGame();
            mgbaModule.current.quitMgba();
          } catch {}
        };
    }, [state]);

    useFrameSkip(mgbaModule);

    useEffect(() => {
      if (!gameLoaded) return;

      const controlEntries = Object.entries(controls);

      for (const [key, value] of controlEntries) {
        const binding =
          value === "Enter"
            ? "Return"
            : value.toLowerCase().replace("arrow", "").replace("key", "");

        mgbaModule.current.bindKey(binding, key);
      }
    }, [controls, gameLoaded]);

    useEffect(() => {
      if (!gameLoaded) return;
      mgbaModule.current.setVolume(volume ?? 1.0);
    }, [gameLoaded, volume]);

    useEffect(() => {
      if (!gameLoaded) return;

      if (paused) {
        mgbaModule.current.pauseGame();
      } else {
        mgbaModule.current.resumeGame();
      }
    }, [gameLoaded, paused]);

    useImperativeHandle(ref, () => {
      return {
        restart: () => mgbaModule.current.quickReload(),
        buttonPress: (key: GbaKey) => mgbaModule.current.buttonPress(key),
        buttonRelease: (key: GbaKey) => mgbaModule.current.buttonUnpress(key),
      };
    });

    return <MgbaCanvas ref={canvas} />;
  }
);
Mgba.displayName = "Mgba";
