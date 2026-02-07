import { RefObject, useEffect } from "react";
import { MgbaEmulatorManager } from "./mgbaEmulator";
import { GbaKey } from "./bindings";

export function useController(
  manager: RefObject<MgbaEmulatorManager | null>
) {
  useEffect(() => {
    let rafId: number | null = null;
    let previouslyPressedButtons = new Set<GbaKey>();

    function raf() {
      const controllers = navigator.getGamepads();
      const currentlyPressed = new Set<GbaKey>();

      for (let controller of controllers) {
        if (!controller) continue;

        if (controller.buttons[1]?.pressed) {
          currentlyPressed.add(GbaKey.A);
        }
        if (controller.buttons[0]?.pressed) {
          currentlyPressed.add(GbaKey.B);
        }
        if (controller.buttons[5]?.pressed) {
          currentlyPressed.add(GbaKey.R);
        }
        if (controller.buttons[4]?.pressed) {
          currentlyPressed.add(GbaKey.L);
        }
        if (controller.buttons[8]?.pressed) {
          currentlyPressed.add(GbaKey.Select);
        }
        if (controller.buttons[9]?.pressed) {
          currentlyPressed.add(GbaKey.Start);
        }
        if (controller.buttons[12]?.pressed) {
          currentlyPressed.add(GbaKey.Up);
        }
        if (controller.buttons[13]?.pressed) {
          currentlyPressed.add(GbaKey.Down);
        }
        if (controller.buttons[14]?.pressed) {
          currentlyPressed.add(GbaKey.Left);
        }
        if (controller.buttons[15]?.pressed) {
          currentlyPressed.add(GbaKey.Right);
        }

        if (controller.axes[0] < -0.5) {
          currentlyPressed.add(GbaKey.Left);
        }
        if (controller.axes[0] > 0.5) {
          currentlyPressed.add(GbaKey.Right);
        }
        if (controller.axes[1] < -0.5) {
          currentlyPressed.add(GbaKey.Up);
        }
        if (controller.axes[1] > 0.5) {
          currentlyPressed.add(GbaKey.Down);
        }
      }

      for (let oldButton of previouslyPressedButtons) {
        if (!currentlyPressed.has(oldButton)) {
          manager.current?.buttonUnpress(oldButton);
        }
      }

      for (let newButton of currentlyPressed) {
        if (!previouslyPressedButtons.has(newButton)) {
          manager.current?.buttonPress(newButton);
        }
      }

      previouslyPressedButtons = currentlyPressed;
      rafId = window.requestAnimationFrame(raf);
    }

    function startPolling() {
      if (rafId === null) {
        rafId = window.requestAnimationFrame(raf);
      }
    }

    function stopPollingIfNoGamepads() {
      const gamepads = navigator.getGamepads();
      const hasGamepad = gamepads.some((gp) => gp !== null);
      if (!hasGamepad && rafId !== null) {
        window.cancelAnimationFrame(rafId);
        rafId = null;
      }
    }

    window.addEventListener("gamepadconnected", startPolling);
    window.addEventListener("gamepaddisconnected", stopPollingIfNoGamepads);

    const gamepads = navigator.getGamepads();
    if (gamepads.some((gp) => gp !== null)) {
      startPolling();
    }

    return () => {
      window.removeEventListener("gamepadconnected", startPolling);
      window.removeEventListener("gamepaddisconnected", stopPollingIfNoGamepads);
      if (rafId !== null) {
        window.cancelAnimationFrame(rafId);
      }
    };
  }, [manager]);
}
