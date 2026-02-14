"use client";

import { useRef } from "react";
import { CodeRunner, CodeRunnerHandle } from "@/components/codeRunner";

const DEFAULT_CODE = `#![no_std]
#![no_main]

use agb::println;

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    println!("Hello from agb!");

    loop {}
}
`;

export function Playground() {
  const runnerRef = useRef<CodeRunnerHandle>(null);

  return (
    <CodeRunner
      ref={runnerRef}
      sourceCode={DEFAULT_CODE}
    />
  );
}
