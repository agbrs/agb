import { useState } from "react";
import styled from "styled-components";

function defaultBindings(): KeyBindings {
  return {
    A: "Z",
    B: "X",
    L: "A",
    R: "S",
    Start: "Enter",
    Select: "Shift",
    Up: "UP",
    Down: "DOWN",
    Left: "LEFT",
    Right: "RIGHT",
  };
}

function defaultActualBindings(): KeyBindings {
  return {
    A: "KeyZ",
    B: "KeyX",
    L: "KeyA",
    R: "KeyS",
    Start: "Enter",
    Select: "ShiftLeft",
    Up: "ArrowUp",
    Down: "ArrowDown",
    Left: "ArrowLeft",
    Right: "ArrowRight",
  };
}

export function DefaultBindingsSet(): Bindings {
  const actual = defaultActualBindings();
  return {
    Actual: actual,
    Displayed: defaultBindings(),
  };
}

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
  margin-top: 10px;
`;

export type KeyBindings = {
  [key in GbaKey]: string;
};

export interface Bindings {
  Displayed: KeyBindings;
  Actual: KeyBindings;
}

function toHumanName(keyName: string) {
  return keyName.replace("Arrow", "");
}

export function BindingsControl({
  bindings,
  setBindings,
  setPaused,
}: {
  bindings: Bindings;
  setBindings: (a: Bindings) => void;
  setPaused: (pause: boolean) => void;
}) {
  const [buttonToChange, setButtonToChange] = useState<GbaKey | null>(null);

  function setKey(key: React.KeyboardEvent) {
    if (buttonToChange === null) return;

    const nextBindings = {
      Displayed: { ...bindings.Displayed },
      Actual: { ...bindings.Actual },
    };

    nextBindings.Displayed[buttonToChange] = toHumanName(key.key).toUpperCase();
    nextBindings.Actual[buttonToChange] = key.code;

    setButtonToChange(null);
    setBindings(nextBindings);
    setPaused(false);
  }

  function onSelectButtonClick(key: GbaKey) {
    setPaused(true);
    setButtonToChange(key);
  }

  return (
    <ButtonWrapper onKeyUp={(evt: React.KeyboardEvent) => setKey(evt)}>
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
}
