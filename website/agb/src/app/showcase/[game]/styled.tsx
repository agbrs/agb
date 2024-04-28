"use client";

import { styled } from "styled-components";

export const ScreenshotsWrapper = styled.div`
  flex: 4;
`;

export const ScreenshotWrapper = styled.div`
  text-align: center;
  img {
    width: 100%;
    width: round(down, 100%, 240px);
    height: auto;
    image-rendering: pixelated;
  }
`;

export const Description = styled.div`
  flex: 5;
  :first-child {
    margin-top: 0;
  }
`;

export const DescriptionAndScreenshots = styled.div`
  display: flex;
  gap: 16px;
`;

export const BackToShowcaseWrapper = styled.div`
  a {
    text-decoration: none;
    color: black;
  }
`;
