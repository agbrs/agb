import { MutableRefObject, useEffect } from "react";
import { mGBAEmulator } from "./vendor/mgba";

export function useFrameSkip(mgbaModule: MutableRefObject<mGBAEmulator>) {
  useEffect(() => {
    let previous: number | undefined = undefined;
    let stopped = false;
    let smoothedFrameTime = 60;

    let totalTime = 0;
    let paused = false;

    function raf(time: DOMHighResTimeStamp) {
      if (previous) {
        const delta = time - previous;

        smoothedFrameTime = (smoothedFrameTime * 3 + delta) / 4;

        const smoothedFrameRate = Math.round(1 / (smoothedFrameTime / 1000));

        totalTime += 1 / smoothedFrameRate;

        if (totalTime >= 1 / 60) {
          totalTime -= 1 / 60;
          if (paused) {
            mgbaModule.current.resumeGame();
            paused = false;
          }
        } else {
          if (!paused) {
            mgbaModule.current.pauseGame();
            paused = true;
          }
        }
      }
      previous = time;

      if (!stopped) {
        window.requestAnimationFrame(raf);
      }
    }

    window.requestAnimationFrame(raf);
    return () => {
      stopped = true;
    };
  }, [mgbaModule]);
}
