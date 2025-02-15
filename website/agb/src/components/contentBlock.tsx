"use client";

import { ReactNode } from "react";
import styled from "styled-components";

const Section = styled.section<{ $color: string }>`
  background-color: ${(props) => props.$color};

  &:last-of-type {
    flex-grow: 1;
  }
`;

const CENTERED_CSS = `
  margin-left: auto;
  margin-right: auto;
  width: 60%;
  min-width: min(95%, 1000px);
`;

export const CenteredBlock = styled.div`
  ${CENTERED_CSS}
`;

const InnerBlock = styled.div<{ $centered?: boolean; $margin: number }>`
  ${(props) => props.$centered && CENTERED_CSS}

  margin-top: ${(props) => props.$margin}px;
  margin-bottom: ${(props) => props.$margin}px;
`;

export function ContentBlock({
  color = "",
  children,
  uncentered = false,
  margin = 40,
}: {
  color?: string;
  uncentered?: boolean;
  children: ReactNode;
  margin?: number;
}) {
  return (
    <Section $color={color}>
      <InnerBlock $centered={!uncentered} $margin={margin}>
        {children}
      </InnerBlock>
    </Section>
  );
}
