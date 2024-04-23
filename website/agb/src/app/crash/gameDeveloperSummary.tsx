import { styled } from "styled-components";

export function GameDeveloperSummary() {
  return (
    <Details>
      <Summary>For game developers</Summary>
      <p>If you don&apos;t want players to be sent to this page, you can:</p>
      <ol>
        <li>
          Configure the backtrace page to point to your own site
          <ul>
            <li>
              Compile with{" "}
              <code>
                AGBRS_BACKTRACE_WEBSITE=&quot;your-website.test/crash#&quot;
                cargo build
              </code>
              .
            </li>
          </ul>
        </li>

        <li>
          Configure the backtrace page to not point to a site at all
          <ul>
            <li>
              Compile with <code>AGBRS_BACKTRACE_WEBSITE= cargo build</code>.
            </li>
          </ul>
        </li>
        <li>
          Not use the backtrace feature
          <ul>
            <li>
              Compile without the default <code>backtrace</code> feature. See{" "}
              <a href="https://doc.rust-lang.org/cargo/reference/features.html#dependency-features">
                the features chapter in the Cargo Book
              </a>{" "}
              for details on how to pick features.
            </li>
          </ul>
        </li>
      </ol>
    </Details>
  );
}

const Details = styled.details`
  margin-top: 10px;
`;

const Summary = styled.summary`
  font-weight: bold;
`;
