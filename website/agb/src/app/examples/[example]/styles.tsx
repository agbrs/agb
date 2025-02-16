"use client";

import Link from "next/link";
import { styled } from "styled-components";

export const BackToExampleLink = styled(Link)`
  text-decoration: none;
  color: black;

  @media (max-width: 500px) {
    display: none;
  }
`;

export const HeightRestricted = styled.div`
  height: 100svh;
  display: flex;
  flex-direction: column;
`;

export const Header = styled.h1`
  margin-top: 0;

  @media (max-width: 500px) {
    margin: 0;
  }
`;
