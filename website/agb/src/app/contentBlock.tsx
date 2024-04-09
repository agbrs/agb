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
  max-width: 60%;

  @media (max-width: 40rem) {
    max-width: 90%;
  }
`;

export const CenteredBlock = styled.div`
  ${CENTERED_CSS}
`;

const InnerBlock = styled.div<{ $centered?: boolean }>`
  ${(props) => props.$centered && CENTERED_CSS}

  margin-top: 40px;
  margin-bottom: 40px;
`;

export const ContentBlock: FC<{
  color?: string;
  uncentered?: boolean;
  children: ReactNode;
}> = ({ color = "", children, uncentered = false }) => (
  <Section $color={color}>
    <InnerBlock $centered={!uncentered}>{children}</InnerBlock>
  </Section>
);
