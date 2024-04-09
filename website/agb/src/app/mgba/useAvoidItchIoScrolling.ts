import { useEffect } from "react";

export const useAvoidItchIoScrolling = () => {
  useEffect(() => {
    const eventHandler = (e: KeyboardEvent) => {
      if ([32, 37, 38, 39, 40].indexOf(e.keyCode) > -1) {
        e.preventDefault();
      }
    };

    window.addEventListener("keydown", eventHandler, false);

    return () => {
      window.removeEventListener("keydown", eventHandler, false);
    };
  }, []);
};
