import { styled } from "styled-components";

export const GameDeveloperSummary = () => {
  return (
    <Details>
      <Summary>For game developers</Summary>
      <p>If you don&apos;t want players to be sent to this page, you can:</p>
      <ol>
        <li>Configure the backtrace page to point to your own site</li>
        <li>Configure the backtrace page to not point to a site at all</li>
        <li>Not use the backtrace feature</li>
      </ol>
    </Details>
  );
};

const Details = styled.details`
  margin-top: 10px;
`;

const Summary = styled.summary`
  font-weight: bold;
`;
