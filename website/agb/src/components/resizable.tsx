import { ReactNode, useEffect, useState } from "react";
import styled from "styled-components";

interface ResizableProps {
  left: ReactNode;
  right: ReactNode;
  className?: string;
  tabContent?: ReactNode;
}

const Box = styled.div<{ $left: number }>`
  @media (min-width: 499px) {
    display: grid;
    grid-template-columns: ${(props) => `${props.$left}fr`} 12px ${(props) =>
        `${1 - props.$left}fr`};
    gap: 8px;
  }

  height: 100%;
  min-height: 0;
`;

const MaxHeight = styled.div<{ $tabOpen: boolean }>`
  height: 100%;
  min-height: 0;
  overflow-y: scroll;

  @media (max-width: 500px) {
    ${(props) => !props.$tabOpen && "display: none;"}
  }
`;

const Center = styled(MaxHeight)`
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: col-resize;

  @media (max-width: 500px) {
    display: none;
  }
`;

const TabContainer = styled.div`
  display: flex;
  flex-direction: row;
  gap: 8px;

  @media (min-width: 499px) {
    display: none;
  }
`;

const Tab = styled.button.attrs<{ $active: boolean }>(() => ({
  type: "button",
}))`
  border: 2px solid black;
  border-bottom: none;
  border-radius: 4px 4px 0 0;
  background-color: ${(props) => (props.$active ? "gray" : "white")};
  flex-grow: 1;
`;

export function Resizable({
  left,
  right,
  className,
  tabContent,
}: ResizableProps) {
  const [edge, setEdge] = useState(0.5);
  const [resizing, setResizing] = useState(false);

  const [openTab, setOpenTab] = useState<"left" | "right">("right");

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
    <div className={className}>
      <TabContainer>
        <Tab $active={openTab === "left"} onClick={() => setOpenTab("left")}>
          Code
        </Tab>
        <Tab $active={openTab === "right"} onClick={() => setOpenTab("right")}>
          Game
        </Tab>
        {tabContent}
      </TabContainer>
      <Box $left={edge}>
        <MaxHeight $tabOpen={openTab === "left"}>{left}</MaxHeight>
        <Center
          $tabOpen
          onMouseDown={(e) => {
            e.preventDefault();
            setResizing(true);
          }}
        >
          <span>⣿</span>
        </Center>
        <MaxHeight $tabOpen={openTab === "right"}>{right}</MaxHeight>
      </Box>
    </div>
  );
}
