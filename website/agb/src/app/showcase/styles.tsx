"use client";

import styled from "styled-components";

export const GameGrid = styled.div`
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
`;

export const GameImage = styled.div`
  img {
    width: 100%;
    width: round(down, 100%, 240px);
    height: auto;
    image-rendering: pixelated;
  }
`;

export const GameDisplay = styled.div`
  width: 600px;
  a {
    text-align: center;
    color: black;
    text-decoration: none;
  }

  h2 {
    margin: 0;
    margin-top: 8px;
  }
`;
