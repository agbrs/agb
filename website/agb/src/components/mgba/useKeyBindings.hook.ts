import { RefObject, useEffect } from "react";
import { MgbaEmulatorManager } from "./mgbaEmulator";
import { KeyBindings } from "./bindings";

export type ControlMode = "always" | "focus";

export function useKeyBindings(
  manager: RefObject<MgbaEmulatorManager | null>,
  canvas: RefObject<HTMLCanvasElement | null>,
  controls: KeyBindings,
  mode: ControlMode
) {
  useEffect(() => {
    if (!canvas.current) return;

    const reverseBindings = new Map(
      Object.entries(controls).map(([a, b]) => [b, a])
    );

    let bindTo: HTMLCanvasElement | Window;
    if (mode === "always") {
      bindTo = window;
    } else {
      if (canvas.current) {
        bindTo = canvas.current;
      } else {
        return;
      }
    }

    for (let control of Object.keys(controls)) {
      manager.current?.buttonUnpress(control);
    }

    function keyDown(event: Event) {
      if (!(event instanceof KeyboardEvent)) return;
      const gbaKey = reverseBindings.get(event.code);
      if (gbaKey) {
        manager.current?.buttonPress(gbaKey);
      }
    }

    function keyUp(event: Event) {
      if (!(event instanceof KeyboardEvent)) return;
      const gbaKey = reverseBindings.get(event.code);
      if (gbaKey) {
        manager.current?.buttonUnpress(gbaKey);
      }
    }

    bindTo.addEventListener("keydown", keyDown);
    bindTo.addEventListener("keyup", keyUp);

    return () => {
      bindTo.removeEventListener("keydown", keyDown);
      bindTo.removeEventListener("keyup", keyUp);
    };
  }, [manager, controls, mode, canvas]);
}
