import { RefObject, useEffect } from "react";
import { mGBAEmulator } from "./vendor/mgba";
import { KeyBindings } from "./bindings";

export type ControlMode = "always" | "focus";

export function useKeyBindings(
  mgbaModule: RefObject<mGBAEmulator | undefined>,
  canvas: RefObject<HTMLCanvasElement | null>,
  controls: KeyBindings,
  mode: ControlMode
) {
  useEffect(() => {
    if (!canvas.current) return;

    const reverseBindings = new Map(
      Object.entries(controls).map(([a, b]) => [b, a])
    );

    let bindTo;
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
      mgbaModule.current?.buttonUnpress(control);
    }

    function keyDown(key: KeyboardEvent) {
      const gbaKey = reverseBindings.get(key.key);
      if (gbaKey) {
        mgbaModule.current?.buttonPress(gbaKey);
      }
    }

    function keyUp(key: KeyboardEvent) {
      const gbaKey = reverseBindings.get(key.key);
      if (gbaKey) {
        mgbaModule.current?.buttonUnpress(gbaKey);
      }
    }

    bindTo.addEventListener("keydown", keyDown as any);
    bindTo.addEventListener("keyup", keyUp as any);

    return () => {
      bindTo.removeEventListener("keydown", keyDown as any);
      bindTo.removeEventListener("keyup", keyUp as any);
    };
  }, [mgbaModule, controls, mode, canvas]);
}
