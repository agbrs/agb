import { ShowcaseGame, shuffle } from "@/app/showcase/games";
import l1 from "./drawing_a_leviathan-0.png";
import l2 from "./drawing_a_leviathan-1.png";
import l3 from "./drawing_a_leviathan-2.png";

const Screenshots = [l1, l2, l3];

export const Leviathan: ShowcaseGame = {
  name: "Drawing a Leviathan",
  developers: shuffle(["Constantin Lietard", "Clara Coolen"]),
  screenshots: Screenshots,
  description: (
    <>
      <p>
        Tasked to document a mythical fish, you will first have to fund your
        expedition to the bottom of the abyss by drawing less mysterious
        creatures.
      </p>
    </>
  ),
  itch: new URL("https://screenshake-farm.itch.io/drawing-a-leviathan"),
};
