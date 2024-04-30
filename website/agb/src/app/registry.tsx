"use client";

import React, { useEffect, useState } from "react";
import { useServerInsertedHTML } from "next/navigation";
import styled, { ServerStyleSheet, StyleSheetManager } from "styled-components";

export default function StyledComponentsRegistry({
  children,
}: {
  children: React.ReactNode;
}) {
  // Only create stylesheet once with lazy initial state
  // x-ref: https://reactjs.org/docs/hooks-reference.html#lazy-initial-state
  const [styledComponentsStyleSheet] = useState(() => new ServerStyleSheet());

  useServerInsertedHTML(() => {
    const styles = styledComponentsStyleSheet.getStyleElement();
    styledComponentsStyleSheet.instance.clearTag();
    return <>{styles}</>;
  });

  if (typeof window !== "undefined") return <>{children}</>;

  return (
    <StyleSheetManager sheet={styledComponentsStyleSheet.instance}>
      {children}
    </StyleSheetManager>
  );
}

const BodyWithPixelRatio = styled.body<{
  $pixel: number;
}>`
  --device-pixel: calc(1px / ${(props) => props.$pixel});
`;

export function BodyPixelRatio({ children }: { children: React.ReactNode }) {
  const [pixel, setPixel] = useState(1);
  useEffect(() => {
    setPixel(window.devicePixelRatio);
  }, []);

  return <BodyWithPixelRatio $pixel={pixel}>{children}</BodyWithPixelRatio>;
}
