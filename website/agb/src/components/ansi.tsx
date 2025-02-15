"use client";

import { Fragment, memo } from "react";
import { ansiToJson, DecorationName } from "anser";
import { styled } from "styled-components";

interface AnsiProps {
  text: string;
}

const Text = styled.span<{
  $fg: string;
  $bg: string;
  $decoration: DecorationName | null;
}>`
  ${(props) => (props.$fg ? `color: rgb(${props.$fg});` : null)}
  ${(props) => (props.$bg ? `background-color: rgb(${props.$bg});` : null)}
  ${(props) => {
    if (props.$decoration == "bold") {
      return "font-weight: bold;";
    }
  }}
`;

const Line = styled.pre`
  margin: 0;
`;

export const Ansi = memo(function Ansi({ text }: AnsiProps) {
  return text
    .replaceAll("\t", "    ")
    .split("\n")
    .map((line, l) => {
      const decoded = ansiToJson(line);

      return (
        <Line key={l}>
          {decoded.map((x, idx) => (
            <Fragment key={idx}>
              <Text $fg={x.fg} $bg={x.bg} $decoration={x.decoration}>
                {x.content}
              </Text>
              {x.clearLine && <br />}
            </Fragment>
          ))}
        </Line>
      );
    });
});
