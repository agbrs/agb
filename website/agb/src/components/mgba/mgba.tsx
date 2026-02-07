"use client";

import { Ref, useEffect, useImperativeHandle, useRef, useState } from "react";
import { LogLevel } from "./vendor/mgba";
import { GbaKey, KeyBindings } from "./bindings";
import { styled } from "styled-components";
import { useController } from "./useController.hook";
import { ControlMode, useKeyBindings } from "./useKeyBindings.hook";
import { MgbaEmulatorManager } from "./mgbaEmulator";

const MgbaCanvas = styled.canvas`
  image-rendering: pixelated;
  aspect-ratio: 240 / 160;
  width: 100%;
  object-fit: contain;
  max-height: 100%;
`;

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
  if (!crypto) return "abcdefg";
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
  controlMode?: ControlMode;
  onLogMessage?: (category: string, level: LogLevel, message: string) => void;
  ref?: Ref<MgbaHandle> | undefined;
}

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

export function Mgba({
  game,
  volume,
  controls,
  paused,
  onLogMessage,
  controlMode = "always",
  ref,
}: MgbaProps) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const manager = useRef<MgbaEmulatorManager | null>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const mgba = new MgbaEmulatorManager();
    manager.current = mgba;

    (async () => {
      try {
        const [gameData, gameName] = await Promise.all([
          downloadGame({ game }),
          generateGameName({ game }),
        ]);
        if (cancelled) return;

        await mgba.init(canvas.current!, gameData, gameName, volume ?? 1.0);
        if (cancelled) return;

        setReady(true);
      } catch (e) {
        console.error("Failed to init mGBA:", e);
      }
    })();

    function beforeUnload() {
      mgba.cleanup();
    }
    window.addEventListener("beforeunload", beforeUnload);

    return () => {
      cancelled = true;
      window.removeEventListener("beforeunload", beforeUnload);
      mgba.cleanup();
      manager.current = null;
      setReady(false);
    };
  }, [game]);

  useEffect(() => {
    if (!ready) return;
    manager.current?.setVolume(volume ?? 1.0);
  }, [ready, volume]);

  useEffect(() => {
    if (!ready) return;
    if (paused) {
      manager.current?.pause();
    } else {
      manager.current?.resume();
    }
  }, [ready, paused]);

  useEffect(() => {
    if (!ready) return;
    manager.current?.setLogListener(onLogMessage);
    return () => {
      manager.current?.setLogListener(undefined);
    };
  }, [ready, onLogMessage]);

  useController(manager);
  useKeyBindings(manager, canvas, controls, controlMode);

  useImperativeHandle(ref, () => ({
    restart: () => manager.current?.restart(),
    buttonPress: (key: GbaKey) => manager.current?.buttonPress(key),
    buttonRelease: (key: GbaKey) => manager.current?.buttonUnpress(key),
  }));

  return <MgbaCanvas tabIndex={-1} ref={canvas} />;
}
