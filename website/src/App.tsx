import React, { useState } from 'react';
import { Mgba } from './mgba';

function App() {

  const [onGame, setOnGame] = useState(false);
  const [volume, setVolume] = useState(1.0);

  return (
    <div>
      {
        onGame && <><Mgba gameUrl="/game.gba" volume={volume} />
          <input type="range" value={volume} min="0" max="1" step="0.05" onChange={(e) => setVolume(Number(e.target.value))}></input></>
      }
      <button onClick={() => setOnGame(!onGame)}>{onGame ? "End Game" : "Start Game"}</button>
    </div>
  );
}

export default App;
