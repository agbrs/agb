import { Metadata } from "next";
import { BacktracePage } from "./backtrace";
import { ContentBlock } from "../contentBlock";

export const metadata: Metadata = {
  title: "agbrs crash backtrace",
};

export default function Backtrace() {
  return <BacktracePage />;
}
