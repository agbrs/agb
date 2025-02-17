import { Ref, useImperativeHandle, useRef, useState } from "react";
import { Game, Mgba, MgbaHandle } from "./mgba";
import {
  BindingsControl,
  DefaultBindingsSet,
  Bindings,
  GbaKey,
} from "./bindings";
import { styled } from "styled-components";
import { useOnKeyUp } from "./useOnKeyUp.hook";
import { useLocalStorage } from "./useLocalStorage.hook";
import { useAvoidItchIoScrolling } from "./useAvoidItchIoScrolling";
import { Slider } from "./Slider";
import { LogLevel } from "./vendor/mgba";
import { ControlMode } from "./useKeyBindings.hook";

const BindingsDialog = styled.dialog`
  border-radius: 5px;
  margin-top: 20px;
  overflow-y: auto;
  max-height: calc(100vh - 100px);
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
  height: 100vh;
  display: flex;
`;

const StartButtonWrapper = styled.button`
  margin: auto;
  font-size: 2em;
  padding: 1em;
  text-transform: uppercase;
  background-color: black;
  color: white;
  border: none;
  aspect-ratio: 240 / 160;
  width: 100%;
  height: 100%;

  &:hover {
    background-color: #222;
    cursor: pointer;
  }
`;

interface MgbaWrapperProps extends Game {
  isPlaying?: boolean;
  onPlayIsClicked?: (isPlaying: boolean) => void;
  onLogMessage?: (category: string, level: LogLevel, message: string) => void;
  ref?: Ref<MgbaHandle> | undefined;
  controlMode?: ControlMode;
}

export function MgbaStandalone(props: MgbaWrapperProps) {
  return (
    <AppContainer>
      <MgbaWrapper {...props} />
    </AppContainer>
  );
}

export function MgbaWrapper({
  game,
  isPlaying = true,
  onPlayIsClicked,
  onLogMessage,
  controlMode,
  ref,
}: MgbaWrapperProps) {
  const [{ volume, bindings }, setState] = useLocalStorage(
    { volume: 1.0, bindings: DefaultBindingsSet() },
    "agbrswebplayer"
  );

  function setVolume(newVolume: number) {
    return setState({ volume: newVolume, bindings });
  }
  function setBindings(newBindings: Bindings) {
    return setState({ volume, bindings: newBindings });
  }

  const [paused, setPaused] = useState(false);

  const [showBindings, setShowBindings] = useState(false);

  const mgbaRef = useRef<MgbaHandle>(null);

  useOnKeyUp("Escape", () => {
    setShowBindings(!showBindings);
  });

  useImperativeHandle(ref, () => ({
    restart: () => mgbaRef.current?.restart(),
    buttonPress: (key: GbaKey) => mgbaRef.current?.buttonPress(key),
    buttonRelease: (key: GbaKey) => mgbaRef.current?.buttonRelease(key),
  }));

  useAvoidItchIoScrolling(controlMode === "always");

  return (
    <>
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
          game={game}
          volume={volume}
          controls={bindings.Actual}
          paused={paused}
          onLogMessage={onLogMessage}
          controlMode={controlMode}
        />
      ) : (
        <StartButton onClick={() => onPlayIsClicked && onPlayIsClicked(true)} />
      )}
    </>
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
    <StartButtonWrapper onClick={onClick}>Touch to start</StartButtonWrapper>
  );
}

export default MgbaWrapper;
