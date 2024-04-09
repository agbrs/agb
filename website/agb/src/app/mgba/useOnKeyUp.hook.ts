import { useEffect } from "react";

export function useOnKeyUp(targetKey: string, callback: () => void) {
  useEffect(() => {
    function downHandler(evnt: KeyboardEvent) {
      if (evnt.key === targetKey) {
        callback();
      }
    }

    window.addEventListener("keyup", downHandler);

    return () => {
      window.removeEventListener("keyup", downHandler);
    };
  }, [callback, targetKey]);
}
