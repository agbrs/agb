import React, { useState } from "react";
import { Mgba } from "./mgba";
import { BindingsControl, DefaultBindingsSet } from "./bindings";

function App() {
  const [volume, setVolume] = useState(1.0);
  const [bindings, setBindings] = useState(DefaultBindingsSet());
  const [paused, setPaused] = useState(false);

  return (
    <div>
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

      <BindingsControl
        bindings={bindings}
        setBindings={setBindings}
        setPaused={setPaused}
      />
    </div>
  );
}

export default App;
