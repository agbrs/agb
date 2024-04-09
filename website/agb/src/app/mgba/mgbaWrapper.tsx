import {
  FC,
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import { Mgba, MgbaHandle } from "./mgba";
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
  font-size: 3em;
  padding: 1em;
  text-transform: uppercase;
  background-color: black;
  color: white;
  border: none;
  border-radius: 0.5em;
  aspect-ratio: 240 / 160;
  width: 100%;
  height: 100%;

  &:hover {
    background-color: #222;
    cursor: pointer;
  }
`;

interface MgbaWrapperProps {
  gameUrl: string;
  startNotPlaying?: boolean;
}

export const MgbaStandalone: FC<MgbaWrapperProps> = (props) => (
  <AppContainer>
    <MgbaWrapper {...props} />
  </AppContainer>
);

export interface MgbaWrapperHandle extends MgbaHandle {
  hardReset: () => void;
}

export const MgbaWrapper = forwardRef<MgbaWrapperHandle, MgbaWrapperProps>(
  ({ gameUrl, startNotPlaying = false }, ref) => {
    const [{ volume, bindings }, setState] = useLocalStorage(
      { volume: 1.0, bindings: DefaultBindingsSet() },
      "agbrswebplayer"
    );

    const [mgbaId, setMgbaId] = useState(0);

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

    useImperativeHandle(ref, () => ({
      restart: () => mgbaRef.current?.restart(),
      buttonPress: (key: GbaKey) => mgbaRef.current?.buttonPress(key),
      buttonRelease: (key: GbaKey) => mgbaRef.current?.buttonRelease(key),
      hardReset: () => setMgbaId((id) => id + 1),
    }));

    useAvoidItchIoScrolling();

    const [isPlaying, setIsPlaying] = useState(!startNotPlaying);

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
            key={mgbaId}
            ref={mgbaRef}
            gameUrl={gameUrl}
            volume={volume}
            controls={bindings.Actual}
            paused={paused}
          />
        ) : (
          <StartButton onClick={() => setIsPlaying(true)} />
        )}
      </>
    );
  }
);
MgbaWrapper.displayName = "MgbaWrapper";

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

export default MgbaWrapper;
