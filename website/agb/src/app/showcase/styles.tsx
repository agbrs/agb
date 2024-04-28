"use client";

import Link from "next/link";
import styled from "styled-components";
import Image from "next/image";

export const GameGrid = styled.div`
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
`;

export const GameImage = styled(Image)`
  width: 100%;
  width: round(down, 100%, 240px);
  height: auto;
  image-rendering: pixelated;
`;

export const GameDisplay = styled(Link)`
  width: 600px;
  text-align: center;
  color: black;
  text-decoration: none;

  h2 {
    margin: 0;
    margin-top: 8px;
  }
`;
