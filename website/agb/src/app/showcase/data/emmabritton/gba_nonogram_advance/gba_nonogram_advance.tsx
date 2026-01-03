import { ShowcaseGame } from "@/app/showcase/games";
import title from "./title_screen.png";
import early from "./early_level.png";
import late from "./late_level.png";

const Screenshots = [title, early, late];

export const NonogramAdvance: ShowcaseGame = {
  name: "Nonogram Advance",
  developers: ["Emma Britton"],
  screenshots: Screenshots,
  description: (
    <>
      <p>Nonogram game with 108 puzzles for the GBA.</p>
      <p>
        Source code is available{" "}
        <a
          href="https://github.com/emmabritton/gba_nonogram_advance"
          target="_blank"
          rel="noopener"
        >
          on GitHub
        </a>
        .
      </p>
    </>
  ),
};
