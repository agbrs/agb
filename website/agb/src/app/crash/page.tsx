import { Metadata } from "next";
import { BacktraceDisplay } from "./backtrace";
import { ContentBlock } from "../contentBlock";

export const metadata: Metadata = {
  title: "agbrs crash backtrace",
};

export default function Crash() {
  return (
    <ContentBlock>
      <h1>agbrs crash backtrace viewer</h1>
      <p>This page will eventually let you view backtraces in the browser.</p>
      <p>
        For now you can copy the backtrace code here and use it with{" "}
        <code>agb-addr2line</code>
      </p>
      <BacktraceDisplay />
    </ContentBlock>
  );
}
