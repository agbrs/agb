"use client";

import Link from "next/link";
import SyntaxHighlighter from "react-syntax-highlighter";
import { styled } from "styled-components";

export const Code = styled(SyntaxHighlighter)`
  font-size: 0.8rem;
`;

export const BackToExampleLink = styled(Link)`
  text-decoration: none;
  color: black;
`;
