import { useRef, useState } from "react";
import { Mgba, MgbaHandle } from "./mgba";
import { BindingsControl, DefaultBindingsSet, Bindings } from "./bindings";
import { styled } from "styled-components";
import { useOnKeyUp } from "./useOnKeyUp.hook";
import { useLocalStorage } from "./useLocalStorage.hook";
import { useAvoidItchIoScrolling } from "./useAvoidItchIoScrolling";
import { Slider } from "./Slider";

const BindingsDialog = styled.dialog`
  border-radius: 5px;
  margin-top: 20px;
`;

const VolumeLabel = styled.label`
  display: flex;
  gap: 10px;
`;

const ActionButton = styled.button`
  width: 100%;
  margin-top: 20px;
`;

const AppContainer = styled.main`
  height: calc(100vh - 20px);
  padding: 10px;
  display: flex;
`;

const StartButtonWrapper = styled.button`
  margin: auto;
  font-size: 5em;
  padding: 1em;
  text-transform: uppercase;
  background-color: black;
  color: white;
  border: none;
  border-radius: 0.5em;

  &:hover {
    background-color: #222;
    cursor: pointer;
  }
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

  useAvoidItchIoScrolling();

  const [isPlaying, setIsPlaying] = useState(false);

  return (
    <AppContainer>
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
      {isPlaying ? (
        <Mgba
          ref={mgbaRef}
          gameUrl="/game.gba"
          volume={volume}
          controls={bindings.Actual}
          paused={paused}
        />
      ) : (
        <StartButton onClick={() => setIsPlaying(true)} />
      )}
    </AppContainer>
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
        <Slider value={volume} onChange={(e) => setVolume(e)} />
      </VolumeLabel>
      <ActionButton onClick={() => setVolume(0)}>Mute</ActionButton>

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

function StartButton({ onClick }: { onClick: () => void }) {
  return (
    <StartButtonWrapper onClick={onClick}>Press to start</StartButtonWrapper>
  );
}

export default App;
