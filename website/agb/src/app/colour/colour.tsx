"use client";

import { ContentBlock } from "../../components/contentBlock";
import { useState } from "react";
import { styled } from "styled-components";

interface Colour {
  r: number;
  g: number;
  b: number;
}

function fromHex(hex: string): Colour {
  if (hex.startsWith("#")) {
    hex = hex.slice(1);
  }

  const c = parseInt(hex, 16);

  return {
    r: (c >> 16) & 255,
    g: (c >> 8) & 255,
    b: c & 255,
  };
}

function toHex(colour: Colour): string {
  const hex = ((colour.r << 16) | (colour.g << 8) | colour.b)
    .toString(16)
    .padStart(6, "0");

  return `#${hex}`;
}

function fromRgb15(colour: number): Colour {
  const r = colour & 31;
  const g = (colour >> 5) & 31;
  const b = (colour >> 10) & 31;

  function upScale(a: number) {
    return a << 3;
  }
  return {
    r: upScale(r),
    g: upScale(g),
    b: upScale(b),
  };
}

function toRgb15(colour: Colour): number {
  const { r, g, b } = colour;
  return ((r >> 3) & 31) | (((g >> 3) & 31) << 5) | (((b >> 3) & 31) << 10);
}

export default function ColourPicker() {
  const [colour, setColour] = useState(fromHex("#FFFFFF"));
  const gbaColour = fromRgb15(toRgb15(colour));

  const hexColour = toHex(colour);
  const gbaHexColour = toHex(gbaColour);
  const gbaU16 = `0x${toRgb15(colour).toString(16)}`;

  function setHexColour(colour: string) {
    setColour(fromHex(colour));
  }

  function setGbaHexColour(colour: string) {
    setColour(fromRgb15(toRgb15(fromHex(colour))));
  }

  return (
    <>
      <ContentBlock color="#9fa6db">
        <h1>agbrs colour converter</h1>
      </ContentBlock>
      <ContentBlock>
        <PickerWrapper>
          <PickerColumn>
            <h2>Regular RGB8</h2>
            <ColourInput
              type="color"
              value={hexColour}
              onChange={(evt) => setHexColour(evt.target.value)}
            />
            <Input
              type="text"
              value={hexColour}
              onChange={(evt) => setHexColour(evt.target.value)}
            />
          </PickerColumn>
          <PickerColumn>
            <h2>GBA RGB5</h2>
            <ColourInput
              type="color"
              value={gbaHexColour}
              onChange={(evt) => setGbaHexColour(evt.target.value)}
            />
            <Input
              type="text"
              value={gbaHexColour}
              onChange={(evt) => setGbaHexColour(evt.target.value)}
            />
            <Input
              type="text"
              value={gbaU16}
              onChange={(evt) =>
                setColour(fromRgb15(parseInt(evt.target.value, 16)))
              }
            />
          </PickerColumn>
        </PickerWrapper>
      </ContentBlock>
    </>
  );
}

const PickerColumn = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  min-width: 40%;
`;

const PickerWrapper = styled.div`
  display: flex;
  justify-content: space-around;
`;

const Input = styled.input`
  width: 100%;
`;

const ColourInput = styled(Input)`
  height: 100px;
  color: #33a012;
`;
