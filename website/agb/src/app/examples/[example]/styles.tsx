"use client";

import Link from "next/link";
import { styled } from "styled-components";

export const BackToExampleLink = styled(Link)`
  text-decoration: none;
  color: black;
`;

export const HeightRestricted = styled.div`
  height: 100vh;
  display: flex;
  flex-direction: column;
`;

export const Header = styled.h1`
  margin-top: 0;
`;
