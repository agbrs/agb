"use client";
import Link from "next/link";
import { styled } from "styled-components";

export const ExternalLink = styled(Link)`
  text-decoration: none;
  color: black;
  background-color: #fad288;
  border: solid #fad288 2px;
  border-radius: 5px;
  padding: 5px 10px;

  &:hover {
    border: solid black 2px;
  }
`;
