import { useEffect } from "react";

export const useOnKeyUp = (targetKey: string, callback: () => void) => {
  useEffect(() => {
    const downHandler = (evnt: KeyboardEvent) => {
      if (evnt.key === targetKey) {
        callback();
      }
    };

    window.addEventListener("keyup", downHandler);

    return () => {
      window.removeEventListener("keyup", downHandler);
    };
  }, [callback, targetKey]);
};
