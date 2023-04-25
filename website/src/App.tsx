import React, { useState } from "react";
import { Mgba } from "./mgba";
import { BindingsControl, DefaultBindingsSet } from "./bindings";

function App() {
  const [onGame, setOnGame] = useState(false);
  const [volume, setVolume] = useState(1.0);
  const [bindings, setBindings] = useState(DefaultBindingsSet());
  const [paused, setPaused] = useState(false);

  return (
    <div>
      {onGame && (
        <>
          <Mgba
            gameUrl="/game.gba"
            volume={volume}
            controls={bindings.Actual}
            paused={paused}
          />
          <input
            type="range"
            value={volume}
            min="0"
            max="1"
            step="0.05"
            onChange={(e) => setVolume(Number(e.target.value))}
          ></input>
        </>
      )}
      <button onClick={() => setOnGame(!onGame)}>
        {onGame ? "End Game" : "Start Game"}
      </button>
      <BindingsControl
        bindings={bindings}
        setBindings={setBindings}
        setPaused={setPaused}
      />
    </div>
  );
}

export default App;
