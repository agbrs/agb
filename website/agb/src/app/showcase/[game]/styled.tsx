"use client";

import { styled } from "styled-components";
import Image, { StaticImageData } from "next/image";

export function Screenshots({
  screenshots,
}: {
  screenshots: StaticImageData[];
}) {
  return (
    <ScreenshotsWrapper>
      {screenshots.map((screenshot) => (
        <Screenshot src={screenshot} alt="" key={screenshot.src} />
      ))}
    </ScreenshotsWrapper>
  );
}

const ScreenshotsWrapper = styled.div`
  flex: 4;
  text-align: center;
`;

const Screenshot = styled(Image)`
  width: 100%;
  width: max(
    round(down, 100%, calc(240 * var(--device-pixel))),
    calc(240 * var(--device-pixel))
  );
  height: auto;
  image-rendering: pixelated;
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

  @media (max-width: 1000px) {
    display: block;
  }
`;

export const BackToShowcaseWrapper = styled.div`
  a {
    text-decoration: none;
    color: black;
  }
`;
