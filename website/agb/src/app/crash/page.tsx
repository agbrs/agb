import { Metadata } from "next";
import { BacktracePage } from "./backtrace";

export const metadata: Metadata = {
  title: "agbrs crash backtrace",
};

export default function Backtrace() {
  return <BacktracePage />;
}
