"use client";

import { FC, ReactNode } from "react";
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

const InnerBlock = styled.div<{ $centered?: boolean }>`
  ${(props) => props.$centered && CENTERED_CSS}

  margin-top: 40px;
  margin-bottom: 40px;
`;

export function ContentBlock({
  color = "",
  children,
  uncentered = false,
}: {
  color?: string;
  uncentered?: boolean;
  children: ReactNode;
}) {
  return (
    <Section $color={color}>
      <InnerBlock $centered={!uncentered}>{children}</InnerBlock>
    </Section>
  );
}
