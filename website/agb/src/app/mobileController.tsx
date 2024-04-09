import { FC, useMemo, useState } from "react";
import styled from "styled-components";
import Image from "next/image";

import TriggerL from "./gba-parts/L.png";
import TriggerR from "./gba-parts/R.png";
import DPad from "./gba-parts/dpad.png";
import ABButtons from "./gba-parts/ab.png";
import Select from "./gba-parts/SELECT.png";
import Start from "./gba-parts/START.png";
import { GbaKey } from "./mgba/bindings";
import { MgbaHandle } from "./mgba/mgba";

const MobileControls = styled.div`
  display: flex;
  gap: 10px;
  justify-content: center;
  align-items: center;
  flex-direction: column;
  margin-bottom: 40px;

  @media (min-width: 500px) {
    display: none;
  }

  touch-action: none;

  img {
    image-rendering: pixelated;
    height: 100%;
    width: unset;
  }
`;

enum MobileControlsSize {
  Big,
  Small,
}

const MobileControlsRow = styled.div<{
  $size: MobileControlsSize;
  $centered?: boolean;
}>`
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;

  height: calc(
    32px * ${(props) => (props.$size === MobileControlsSize.Big ? 6 : 3)}
  );
  ${(props) => props.$centered && `justify-content: center;`}
`;

function useSimpleButton(mgba: MgbaHandle, button: GbaKey) {
  return useMemo(() => {
    return {
      onTouchStart: () => {
        mgba?.buttonPress(button);
      },
      onTouchEnd: () => {
        mgba?.buttonRelease(button);
      },
    };
  }, [button, mgba]);
}

function relativeTouch(touch: Touch) {
  const target = (touch.target as Element).getBoundingClientRect();

  const touchPoint = { x: touch.clientX, y: touch.clientY };
  const targetArea = {
    x: target.left,
    y: target.top,
    width: target.width,
    height: target.height,
  };

  const relativePosition = {
    x: (touchPoint.x - targetArea.x) / targetArea.width,
    y: (touchPoint.y - targetArea.y) / targetArea.height,
  };

  return relativePosition;
}

function useDpadTouch(mgba: MgbaHandle) {
  const [previouslyPressedButtons, setTouchedButtons] = useState<Set<GbaKey>>(
    new Set()
  );

  return useMemo(() => {
    function updateDpad(touches: TouchList) {
      const currentlyPressed = new Set<GbaKey>();

      for (let touch of touches) {
        const relative = relativeTouch(touch);
        const touchedBox = {
          x: relative.x * 3,
          y: relative.y * 3,
        };

        if (touchedBox.y <= 1) {
          currentlyPressed.add(GbaKey.Up);
        }

        if (touchedBox.y >= 2) {
          currentlyPressed.add(GbaKey.Down);
        }

        if (touchedBox.x <= 1) {
          currentlyPressed.add(GbaKey.Left);
        }

        if (touchedBox.x >= 2) {
          currentlyPressed.add(GbaKey.Right);
        }
      }

      for (let oldButton of previouslyPressedButtons) {
        if (!currentlyPressed.has(oldButton)) {
          mgba.buttonRelease(oldButton);
        }
      }

      for (let newButton of currentlyPressed) {
        if (!previouslyPressedButtons.has(newButton)) {
          mgba.buttonPress(newButton);
        }
      }

      setTouchedButtons(currentlyPressed);
    }

    return {
      onTouchStart: (event: React.TouchEvent) =>
        updateDpad(event.nativeEvent.targetTouches),
      onTouchEnd: (event: React.TouchEvent) =>
        updateDpad(event.nativeEvent.targetTouches),
      onTouchMove: (event: React.TouchEvent) =>
        updateDpad(event.nativeEvent.targetTouches),
    };
  }, [mgba, previouslyPressedButtons]);
}

function useAbTouch(mgba: MgbaHandle) {
  const [previouslyPressedButtons, setTouchedButtons] = useState<Set<GbaKey>>(
    new Set()
  );

  return useMemo(() => {
    function updateAbButtons(touches: TouchList) {
      const currentlyPressed = new Set<GbaKey>();

      for (let touch of touches) {
        const relative = relativeTouch(touch);

        const aIsPressed = relative.x > relative.y;

        currentlyPressed.add(aIsPressed ? GbaKey.A : GbaKey.B);
      }

      for (let oldButton of previouslyPressedButtons) {
        if (!currentlyPressed.has(oldButton)) {
          mgba.buttonRelease(oldButton);
        }
      }

      for (let newButton of currentlyPressed) {
        if (!previouslyPressedButtons.has(newButton)) {
          mgba.buttonPress(newButton);
        }
      }

      setTouchedButtons(currentlyPressed);
    }

    return {
      onTouchStart: (event: React.TouchEvent) =>
        updateAbButtons(event.nativeEvent.targetTouches),
      onTouchEnd: (event: React.TouchEvent) =>
        updateAbButtons(event.nativeEvent.targetTouches),
      onTouchMove: (event: React.TouchEvent) =>
        updateAbButtons(event.nativeEvent.targetTouches),
    };
  }, [mgba, previouslyPressedButtons]);
}

export function MobileController({ mgba }: { mgba: MgbaHandle }) {
  return (
    <MobileControls onContextMenu={(evt) => evt.preventDefault()}>
      <MobileControlsRow $size={MobileControlsSize.Small}>
        <Image
          {...useSimpleButton(mgba, GbaKey.L)}
          src={TriggerL}
          alt="L trigger"
        />
        <Image
          {...useSimpleButton(mgba, GbaKey.R)}
          src={TriggerR}
          alt="R trigger"
        />
      </MobileControlsRow>
      <MobileControlsRow $size={MobileControlsSize.Big}>
        <Image
          {...useDpadTouch(mgba)}
          src={DPad}
          alt="Directional pad (Dpad)"
        />
        <Image {...useAbTouch(mgba)} src={ABButtons} alt="A / B buttons" />
      </MobileControlsRow>
      <MobileControlsRow $size={MobileControlsSize.Small}>
        <Image
          {...useSimpleButton(mgba, GbaKey.Select)}
          src={Select}
          alt="Select button"
        />
        <Image
          {...useSimpleButton(mgba, GbaKey.Start)}
          src={Start}
          alt="Start button"
        />
      </MobileControlsRow>
      <MobileControlsRow $size={MobileControlsSize.Small} $centered>
        <button onClick={() => mgba.restart()}>Restart</button>
      </MobileControlsRow>
    </MobileControls>
  );
}
