import { FC, useEffect, useRef, useState } from "react";
import mGBA from "./vendor/mgba";

type Module = any;

interface MgbaProps {
    gameUrl: string,
    volume?: Number,
}

enum MgbaState {
    Uninitialised,
    Initialising,
    Initialised,
}

const MGBA_ROM_DIRECTORY = "/data/games";


export const Mgba: FC<MgbaProps> = ({ gameUrl, volume }) => {

    const canvas = useRef(null);
    const mgbaModule = useRef<Module>({});

    const [state, setState] = useState(MgbaState.Uninitialised);

    useEffect(() => {
        if (state !== MgbaState.Initialised) return;
        (async () => {
            const game = await fetch(gameUrl);
            const gameData = await game.arrayBuffer();

            const gamePath = `${MGBA_ROM_DIRECTORY}/${gameUrl}`;
            mgbaModule.current.FS.writeFile(gamePath, new Uint8Array(gameData));
            mgbaModule.current.loadGame(gamePath);
        })()
    }, [state, gameUrl]);

    // init mgba
    useEffect(() => {
        (async () => {

            if (canvas === null) return;
            if (state !== MgbaState.Uninitialised) return;

            setState(MgbaState.Initialising);

            mgbaModule.current = {
                canvas: canvas.current,
                locateFile: (file: string) => {
                    if (file === "mgba.wasm") {
                        return "/vendor/mgba.wasm";
                    }
                    return file;
                }
            };

            mGBA(mgbaModule.current).then((module: Module) => {
                mgbaModule.current = module;
                module.FSInit();
                setState(MgbaState.Initialised);
            });
        }
        )();

        if (state === MgbaState.Initialised)
            return () => {
                try {
                    mgbaModule.current.quitGame();
                    mgbaModule.current.quitMgba();
                } catch { }
            };
    }, [state]);

    useEffect(() => {
        if (state !== MgbaState.Initialised) return;
        mgbaModule.current.setVolume(volume ?? 1.0);
    }, [state, volume]);

    return <>

        <canvas ref={canvas}></canvas>
        <button onClick={() => {
            if (state !== MgbaState.Initialised) return;
            mgbaModule.current.saveState(0);
        }}>Save State</button>
        <button onClick={() => {
            if (state !== MgbaState.Initialised) return;
            mgbaModule.current.loadState(0);
        }}>Load State</button>
        <button onClick={() => {
            if (state !== MgbaState.Initialised) return;
            mgbaModule.current.quickReload(0);
        }}>Restart</button>

    </>;
};