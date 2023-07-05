import { useRef, useLayoutEffect, useEffect } from "react";

export const useLocalStorage = <T>(currentValue: T, appName: string): T => {
  const initialValue = useRef<T>();

  const isFirstRun = !initialValue.current;

  useLayoutEffect(() => {
    if (!initialValue.current) {
      try {
        const storageValue = localStorage.getItem(appName);
        if (storageValue) {
          initialValue.current = JSON.parse(storageValue);
        } else {
          initialValue.current = currentValue;
        }
      } catch {
        initialValue.current = currentValue;
      }
    }
  }, []);

  useEffect(() => {
    try {
      if (initialValue.current && currentValue) {
        localStorage.setItem(appName, JSON.stringify(currentValue));
      }
    } catch {}
  }, [currentValue]);

  if (isFirstRun) {
    return initialValue.current ?? currentValue;
  } else {
    return currentValue;
  }
};
