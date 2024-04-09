import { useEffect, useState } from "react";

export function useClientValue<T>(fn: () => T) {
    const [value, setValue] = useState<T>();
    useEffect(() => {
        setValue(fn());
    }, [fn]);

    return value;
}