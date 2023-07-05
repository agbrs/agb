import { useRef, useState } from "react";
import { Mgba, MgbaHandle } from "./mgba";
import { BindingsControl, DefaultBindingsSet, Bindings } from "./bindings";
import { styled } from "styled-components";
import { useOnKeyUp } from "./useOnKeyUp.hook";
import { useLocalStorage } from "./useLocalStorage.hook";

const BindingsDialog = styled.dialog`
  border-radius: 5px;
  margin-top: 20px;
`;

const VolumeLabel = styled.label`
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
`;

const ActionButton = styled.button`
  width: 100%;
  margin-top: 20px;
`;

function App() {
  const [{ volume, bindings }, setState] = useLocalStorage(
    { volume: 1.0, bindings: DefaultBindingsSet() },
    "agbrswebplayer"
  );

  const setVolume = (newVolume: number) =>
    setState({ volume: newVolume, bindings });
  const setBindings = (newBindings: Bindings) =>
    setState({ volume, bindings: newBindings });

  const [paused, setPaused] = useState(false);

  const [showBindings, setShowBindings] = useState(false);

  const mgbaRef = useRef<MgbaHandle>(null);

  useOnKeyUp("Escape", () => {
    setShowBindings(!showBindings);
  });

  return (
    <div>
      {showBindings && (
        <BindingsWindow
          bindings={bindings}
          setBindings={setBindings}
          setPaused={setPaused}
          volume={volume}
          setVolume={setVolume}
          hide={() => setShowBindings(false)}
          restart={() => mgbaRef.current?.restart()}
        />
      )}
      <Mgba
        ref={mgbaRef}
        gameUrl="/game.gba"
        volume={volume}
        controls={bindings.Actual}
        paused={paused}
      />
    </div>
  );
}

function BindingsWindow({
  bindings,
  setBindings,
  setPaused,
  volume,
  setVolume,
  hide,
  restart,
}: {
  bindings: Bindings;
  setBindings: (b: Bindings) => void;
  setPaused: (paused: boolean) => void;
  volume: number;
  setVolume: (v: number) => void;
  hide: () => void;
  restart: () => void;
}) {
  return (
    <BindingsDialog open onClose={hide}>
      <VolumeLabel>
        Volume:
        <input
          type="range"
          value={volume}
          min="0"
          max="1"
          step="0.05"
          onChange={(e) => {
            console.log("e.target.value", e.target.value);
            console.log("volume", volume);
            setVolume(Number(e.target.value));
          }}
        />
      </VolumeLabel>
      <BindingsControl
        bindings={bindings}
        setBindings={setBindings}
        setPaused={setPaused}
      />
      <ActionButton onClick={restart}>Restart</ActionButton>
      <ActionButton onClick={hide}>Close</ActionButton>
    </BindingsDialog>
  );
}

export default App;
