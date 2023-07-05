import { FC, useState } from "react";
import styled from "styled-components";

const DefaultBindings = (): KeyBindings => {
  return {
    A: "Z",
    B: "X",
    L: "A",
    R: "S",
    Start: "ENTER",
    Select: "SHIFT",
    Up: "UP",
    Down: "DOWN",
    Left: "LEFT",
    Right: "RIGHT",
  };
};

export const DefaultBindingsSet = (): Bindings => ({
  Actual: DefaultBindings(),
  Displayed: DefaultBindings(),
});

export enum GbaKey {
  A = "A",
  B = "B",
  L = "L",
  R = "R",
  Up = "Up",
  Down = "Down",
  Left = "Left",
  Right = "Right",
  Start = "Start",
  Select = "Select",
}

const BindingsOrder = [
  GbaKey.A,
  GbaKey.B,
  GbaKey.L,
  GbaKey.R,
  GbaKey.Up,
  GbaKey.Down,
  GbaKey.Left,
  GbaKey.Right,
  GbaKey.Start,
  GbaKey.Select,
];

interface SelectButtonProps {
  selected: boolean;
}

const SelectButton = styled.button<SelectButtonProps>`
  grid-column: 1;
`;

const ButtonWrapper = styled.div`
  display: grid;
`;

export type KeyBindings = {
  [key in GbaKey]: string;
};

export interface Bindings {
  Displayed: KeyBindings;
  Actual: KeyBindings;
}

const toHumanName = (keyName: string) => {
  return keyName.replace("Arrow", "");
};

export const BindingsControl: FC<{
  bindings: Bindings;
  setBindings: (a: Bindings) => void;
  setPaused: (pause: boolean) => void;
}> = ({ bindings, setBindings, setPaused }) => {
  const [buttonToChange, setButtonToChange] = useState<GbaKey | null>(null);

  const setKey = (key: string) => {
    if (buttonToChange === null) return;

    const nextBindings = {
      Displayed: { ...bindings.Displayed },
      Actual: { ...bindings.Actual },
    };

    nextBindings.Displayed[buttonToChange] = toHumanName(key).toUpperCase();
    nextBindings.Actual[buttonToChange] = key;

    setButtonToChange(null);
    setBindings(nextBindings);
    setPaused(false);
  };

  const onSelectButtonClick = (key: GbaKey) => {
    setPaused(true);
    setButtonToChange(key);
  };

  return (
    <ButtonWrapper onKeyUp={(evt: React.KeyboardEvent) => setKey(evt.key)}>
      {BindingsOrder.map((x) => (
        <SelectButton
          onClick={() => onSelectButtonClick(x)}
          key={x}
          selected={buttonToChange === x}
        >
          {buttonToChange === x
            ? `Change ${x}`
            : `${x}: ${bindings.Displayed[x]}`}
        </SelectButton>
      ))}
    </ButtonWrapper>
  );
};
