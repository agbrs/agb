"use client";

import Link from "next/link";
import styled from "styled-components";
import Image from "next/image";

export const GameGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, min(100vw, 600px));
  justify-content: center;
  gap: 48px;
`;

export const GameImage = styled(Image)`
  width: 100%;
  width: max(
    round(down, 100%, calc(240 * var(--device-pixel))),
    min(calc(240 * var(--device-pixel)), 100vw)
  );
  height: auto;
  image-rendering: pixelated;
`;

export const GameDisplay = styled(Link)`
  width: 100%;
  text-align: center;
  color: black;
  text-decoration: none;

  h2 {
    margin: 0;
    margin-top: 8px;
  }
`;
