"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import {
  ActionButton,
  CodeRunner,
  CodeRunnerHandle,
} from "@/components/codeRunner";

const PLAYGROUND_URL = process.env.NEXT_PUBLIC_PLAYGROUND_URL;

const DEFAULT_CODE = `#![no_std]
#![no_main]

use agb::println;

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    println!("Hello from agb!");

    loop {}
}
`;

type ShareStatus = "idle" | "copied" | "error";

const SHARE_BUTTON_TEXT: Record<ShareStatus, string> = {
  copied: "Copied!",
  error: "Failed to share",
  idle: "Share",
};

async function loadGist(id: string): Promise<string> {
  const response = await fetch(`${PLAYGROUND_URL}/gist/${id}`);
  if (!response.ok) {
    throw new Error("Failed to load gist");
  }
  const data = await response.json();
  return data.code;
}

export function Playground() {
  const [initialCode, setInitialCode] = useState<string | null>(null);
  const [shareStatus, setShareStatus] = useState<ShareStatus>("idle");
  const runnerRef = useRef<CodeRunnerHandle>(null);

  useEffect(() => {
    const hash = window.location.hash.slice(1);
    if (hash.startsWith("gist=")) {
      loadGist(hash.slice("gist=".length))
        .then((code) => {
          setInitialCode(code);
          setTimeout(() => runnerRef.current?.build(), 100);
        })
        .catch(() => {
          setInitialCode(DEFAULT_CODE);
        });
    } else {
      setInitialCode(DEFAULT_CODE);
    }
  }, []);

  const handleShare = useCallback(async () => {
    if (!runnerRef.current) return;
    const code = runnerRef.current.getCode();

    try {
      const response = await fetch(`${PLAYGROUND_URL}/gist`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ code }),
      });

      if (!response.ok) {
        throw new Error("Gist creation failed");
      }

      const data = await response.json();
      window.location.hash = `gist=${data.id}`;

      await navigator.clipboard.writeText(window.location.href);
      setShareStatus("copied");
    } catch {
      setShareStatus("error");
    }
    setTimeout(() => setShareStatus("idle"), 2000);
  }, []);

  if (initialCode === null) return null;

  return (
    <CodeRunner
      ref={runnerRef}
      sourceCode={initialCode}
      extraButtons={({ isPending }) => (
        <ActionButton onClick={handleShare} disabled={isPending}>
          {SHARE_BUTTON_TEXT[shareStatus]}
        </ActionButton>
      )}
    />
  );
}
