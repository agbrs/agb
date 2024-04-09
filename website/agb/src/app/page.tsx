"use client";

import styled from "styled-components";
import { CenteredBlock, ContentBlock } from "./contentBlock";
import MgbaWrapper from "./mgba/mgbaWrapper";
import Image from "next/image";

import left from "./gba-parts/left.png";
import right from "./gba-parts/right.png";
import { MobileController } from "./mobileController";
import { useMemo, useRef, useState } from "react";
import { GbaKey } from "./mgba/bindings";
import { useClientValue } from "./useClientValue.hook";
import { MgbaHandle } from "./mgba/mgba";

const ExternalLink = styled.a`
  text-decoration: none;
  color: black;
  background-color: #fad288;
  border: solid #fad288 2px;
  border-radius: 5px;
  padding: 5px 10px;
`;

const HelpLinks = styled.div`
  display: flex;
  justify-content: space-around;
`;

const GameDisplay = styled.div`
  height: min(calc(100vw / 1.5), 40vh);
  max-width: 100vw;
  margin-top: 20px;
  overflow: hidden;
`;

const GamePanelWrapper = styled.div`
  display: flex;
  justify-content: center;
  align-items: end;
  height: 100%;
`;

const GameDisplayWindow = styled.div`
  border: 0;
  height: 100%;
  max-width: 100vw;
  aspect-ratio: 240 / 160;
`;

const GameSide = styled.div`
  aspect-ratio: 15 / 31;
  height: 100%;

  img {
    height: 100%;
    width: 100%;
    image-rendering: pixelated;
  }
`;

const isTouchScreen = () => navigator.maxTouchPoints > 1;

function shouldStartPlaying(isTouchScreen: boolean | undefined) {
  if (isTouchScreen === undefined) return false;
  return !isTouchScreen;
}

const MgbaWithControllerSides = () => {
  const mgba = useRef<MgbaHandle>(null);

  const mgbaHandle = useMemo(
    () => ({
      restart: () => mgba.current?.restart(),
      buttonPress: (key: GbaKey) => mgba.current?.buttonPress(key),
      buttonRelease: (key: GbaKey) => mgba.current?.buttonRelease(key),
    }),
    []
  );

  const [isPlaying, setIsPlaying] = useState<boolean>();
  const shouldUseTouchScreenInput = useClientValue(isTouchScreen);

  const playEmulator =
    isPlaying ?? shouldStartPlaying(shouldUseTouchScreenInput);

  return (
    <>
      <GameDisplay>
        <GamePanelWrapper>
          <GameSide>
            <Image src={left} alt="" />
          </GameSide>
          <GameDisplayWindow>
            <MgbaWrapper
              gameUrl="combo.gba.gz"
              ref={mgba}
              isPlaying={playEmulator}
              setIsPlaying={setIsPlaying}
            />
          </GameDisplayWindow>
          <GameSide>
            <Image src={right} alt="" />
          </GameSide>
        </GamePanelWrapper>
      </GameDisplay>
      {shouldUseTouchScreenInput ? (
        <MobileController mgba={mgbaHandle} />
      ) : (
        <CenteredBlock>
          <p>
            Press escape to open the menu where you can view or change controls
            and restart the game. The game provided is a combination of multiple
            Game Boy Advance games made using agb, you can press left or right
            on the main menu to switch game.
          </p>
        </CenteredBlock>
      )}
    </>
  );
};
export default function Home() {
  return (
    <>
      <ContentBlock>
        <h1>agb - a rust framework for making Game Boy Advance games</h1>
      </ContentBlock>
      <ContentBlock uncentered>
        <MgbaWithControllerSides />
      </ContentBlock>
      <ContentBlock color="#f5755e">
        <HelpLinks>
          <ExternalLink href="https://github.com/agbrs/agb">
            GitHub
          </ExternalLink>
          <ExternalLink href="book/">Book</ExternalLink>
          <ExternalLink href="https://docs.rs/agb/latest/agb/">
            Docs
          </ExternalLink>
        </HelpLinks>
      </ContentBlock>
    </>
  );
}
