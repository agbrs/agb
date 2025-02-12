"use client";

import styled from "styled-components";

export const Flex = styled.div<{
  $v?: boolean;
  $gapR?: string;
  $gapC?: string;
  $grow?: number;
}>`
  display: flex;
  row-gap: ${(props) => props.$gapR ?? "0px"};
  column-gap: ${(props) => props.$gapC ?? "0px"};
  flex-direction: ${(props) => (props.$v ? "column" : "row")};
  ${(props) => props.$grow && `flex-grow: ${props.$grow}`}
`;
