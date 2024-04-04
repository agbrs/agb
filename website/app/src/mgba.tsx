import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import mGBA from "./vendor/mgba";
import { KeyBindings } from "./bindings";
import { styled } from "styled-components";

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
}

export const Mgba = forwardRef<MgbaHandle, MgbaProps>(
  ({ gameUrl, volume, controls, paused }, ref) => {
    const canvas = useRef(null);
    const mgbaModule = useRef<Module>({});

    const [state, setState] = useState(MgbaState.Uninitialised);
    const [gameLoaded, setGameLoaded] = useState(false);

    useEffect(() => {
      if (state !== MgbaState.Initialised) return;
      (async () => {
        const game = await fetch(gameUrl);
        const gameData = await game.arrayBuffer();

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
      };
    });

    return <MgbaCanvas ref={canvas} />;
  }
);
