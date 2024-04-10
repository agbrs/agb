import { MutableRefObject, useEffect } from "react";
import { mGBAEmulator } from "./vendor/mgba";
import { GbaKey } from "./bindings";

export function useController(mgbaModule: MutableRefObject<mGBAEmulator>) {
    useEffect(() => {
        let stopped = false;


        let previouslyPressedButtons = new Set<GbaKey>();

        function raf(time: DOMHighResTimeStamp) {

            const controllers = navigator.getGamepads();
            const currentlyPressed = new Set<GbaKey>();
            for (let controller of controllers) {
                if (!controller) continue;


                if (controller.buttons[1].pressed) {
                    currentlyPressed.add(GbaKey.A);
                }
                if (controller.buttons[0].pressed) {
                    currentlyPressed.add(GbaKey.B);
                }
                if (controller.buttons[5].pressed) {
                    currentlyPressed.add(GbaKey.R);
                }
                if (controller.buttons[4].pressed) {
                    currentlyPressed.add(GbaKey.L);
                }

                if (controller.buttons[8].pressed) {
                    currentlyPressed.add(GbaKey.Select);
                }

                if (controller.buttons[9].pressed) {
                    currentlyPressed.add(GbaKey.Start);
                }

                if (controller.buttons[12].pressed) {
                    currentlyPressed.add(GbaKey.Up);
                }
                if (controller.buttons[13].pressed) {
                    currentlyPressed.add(GbaKey.Down);
                }
                if (controller.buttons[14].pressed) {
                    currentlyPressed.add(GbaKey.Left);
                }
                if (controller.buttons[15].pressed) {
                    currentlyPressed.add(GbaKey.Right);
                }

                if (controller.axes[0] < -.5) {
                    currentlyPressed.add(GbaKey.Left);
                }
                if (controller.axes[0] > .5) {
                    currentlyPressed.add(GbaKey.Right);
                }
                if (controller.axes[1] < -.5) {
                    currentlyPressed.add(GbaKey.Up);
                }
                if (controller.axes[1] > .5) {
                    currentlyPressed.add(GbaKey.Down);
                }


            }

            for (let oldButton of previouslyPressedButtons) {
                if (!currentlyPressed.has(oldButton)) {
                    mgbaModule.current.buttonUnpress(oldButton);
                }
            }

            for (let newButton of currentlyPressed) {
                if (!previouslyPressedButtons.has(newButton)) {
                    mgbaModule.current.buttonPress(newButton);
                }
            }

            previouslyPressedButtons = currentlyPressed;

            if (!stopped) {
                window.requestAnimationFrame(raf);
            }
        }

        function gamepadConnectedEvent() {

        }

        window.addEventListener("gamepadconnected", gamepadConnectedEvent);

        window.requestAnimationFrame(raf);
        return () => { stopped = true; window.removeEventListener("gamepadconnected", gamepadConnectedEvent); };
    }, [mgbaModule]);
}