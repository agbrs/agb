import { ReactNode, useEffect, useState } from "react";
import styled from "styled-components";

interface ResizableProps {
  left: ReactNode;
  right: ReactNode;
  className?: string;
}

const Box = styled.div<{ $left: number }>`
  display: grid;
  grid-template-columns: ${(props) => `${props.$left}fr`} 12px ${(props) =>
      `${1 - props.$left}fr`};
  gap: 8px;
`;

const MaxHeight = styled.div`
  height: 100%;
  min-height: 0;
  overflow-y: scroll;
`;

const Center = styled(MaxHeight)`
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: col-resize;
`;

export function Resizable({ left, right, className }: ResizableProps) {
  const [edge, setEdge] = useState(0.5);
  const [resizing, setResizing] = useState(false);

  useEffect(() => {
    if (!resizing) return;
    function onMove(e: MouseEvent) {
      const x = e.clientX;
      const width = window.innerWidth;
      setEdge(x / width);
    }

    function onRelease(e: MouseEvent) {
      setResizing(false);
    }

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onRelease);

    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onRelease);
    };
  }, [resizing]);

  return (
    <Box $left={edge} className={className}>
      <MaxHeight>{left}</MaxHeight>
      <Center
        onMouseDown={(e) => {
          e.preventDefault();
          setResizing(true);
        }}
      >
        <span>⣿</span>
      </Center>
      <MaxHeight>{right}</MaxHeight>
    </Box>
  );
}
