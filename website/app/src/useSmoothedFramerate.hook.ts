import { useEffect, useState } from "react";


export const useSmoothedFramerate = (): number => {

    const [smoothedFrameTime, setSmoothedFrameTime] = useState(60);

    useEffect(() => {

        let previous: number | undefined = undefined;
        let stopped = false;

        const raf = (time: DOMHighResTimeStamp) => {
            if (previous) {
                let delta = time - previous;

                setSmoothedFrameTime((time) => (time * 3 + delta) / 4);
            }
            previous = time;


            if (!stopped) {
                window.requestAnimationFrame(raf);
            }
        }

        window.requestAnimationFrame(raf);

        return () => { stopped = true; }

    }, []);


    return Math.round(1 / (smoothedFrameTime / 1000));
}