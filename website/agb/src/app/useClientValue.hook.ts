import { useEffect, useState } from "react";

export const useClientValue = <T,>(fn: () => T) => {
    const [value, setValue] = useState<T>();
    useEffect(() => {
        setValue(fn());
    }, [fn]);

    return value;
}