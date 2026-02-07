import mGBA, { mGBAEmulator, LogLevel } from "./vendor/mgba";
import { GbaKey } from "./bindings";

const MGBA_ROM_DIRECTORY = "/data/games";
const MGBA_SAVE_DIRECTORY = "/data/saves";
const SAVE_STORAGE_KEY = "agbrswebplayer/savegames";

interface SaveGameStore {
  [gameName: string]: number[];
}

type LogListener = (category: string, level: LogLevel, message: string) => void;

export class MgbaEmulatorManager {
  private module: mGBAEmulator | null = null;
  private abortController = new AbortController();
  private gameName: string = "";
  private logListener: LogListener | null = null;

  async init(
    canvas: HTMLCanvasElement,
    gameData: ArrayBuffer,
    gameName: string,
    volume: number
  ): Promise<void> {
    this.gameName = gameName;
    const { signal } = this.abortController;

    const module = await mGBA({ canvas });
    if (signal.aborted) {
      return;
    }

    module.toggleInput(false);
    await module.FSInit();
    if (signal.aborted) {
      return;
    }

    await module.FSSync();
    if (signal.aborted) {
      return;
    }

    this.module = module;

    this.restoreSave(gameName);

    const gamePath = `${MGBA_ROM_DIRECTORY}/${gameName}.gba`;
    module.FS.writeFile(gamePath, new Uint8Array(gameData));
    module.loadGame(gamePath);
    module.setVolume(volume);
  }

  private restoreSave(gameName: string): void {
    try {
      const raw = localStorage.getItem(SAVE_STORAGE_KEY);
      if (!raw) return;

      const store: SaveGameStore = JSON.parse(raw);
      const save = store[gameName];
      if (!save) return;

      const savePath = `${MGBA_SAVE_DIRECTORY}/${gameName}.sav`;
      this.module?.FS.writeFile(savePath, new Uint8Array(save));
    } catch {}
  }

  persistSave(): void {
    if (!this.module || !this.gameName) return;

    try {
      const save = this.module.getSave();
      if (!save) return;

      const raw = localStorage.getItem(SAVE_STORAGE_KEY);
      const store: SaveGameStore = raw ? JSON.parse(raw) : {};
      store[this.gameName] = [...save];
      localStorage.setItem(SAVE_STORAGE_KEY, JSON.stringify(store));
    } catch {}
  }

  setVolume(v: number): void {
    this.module?.setVolume(v);
  }

  pause(): void {
    this.module?.pauseGame();
  }

  resume(): void {
    this.module?.resumeGame();
  }

  restart(): void {
    this.module?.quickReload();
  }

  buttonPress(key: GbaKey | string): void {
    this.module?.buttonPress(key);
  }

  buttonUnpress(key: GbaKey | string): void {
    this.module?.buttonUnpress(key);
  }

  setLogListener(fn: LogListener | undefined): void {
    if (this.logListener) {
      this.module?.removeLogListener(this.logListener);
      this.logListener = null;
    }

    if (fn) {
      this.logListener = fn;
      this.module?.addLogListener(fn);
    }
  }

  cleanup(): void {
    this.abortController.abort();

    if (!this.module) return;

    if (this.logListener) {
      this.module.removeLogListener(this.logListener);
      this.logListener = null;
    }

    try {
      this.module.quitGame();
    } catch {}

    this.persistSave();

    try {
      this.module.quitMgba();
    } catch {}

    this.module = null;
  }
}
