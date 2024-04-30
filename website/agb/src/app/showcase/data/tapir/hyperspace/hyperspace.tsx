import { ShowcaseGame, shuffle } from "@/app/showcase/games";
import h1 from "./hyperspace-roll-0.png";
import h2 from "./hyperspace-roll-1.png";

const Screenshots = [h1, h2];

export const Hyperspace: ShowcaseGame = {
  name: "Hyperspace Roll",
  developers: shuffle(["Corwin Kuiper", "Gwilym Inzani", "Sam Williams"]),
  screenshots: Screenshots,
  description: (
    <>
      <p>
        Get through as many levels as possible in this space themed, dice
        rolling roguelike.
      </p>
      <p>
        Build up powerful combos to defeat enemies which keep getting stronger.
        Slowly acquire more dice and upgrade them in order to handle the
        increasing strength of the enemies you face.
      </p>

      <p>
        Hyperspace Roll was influenced by great games such as Slay the Spire,
        FTL and the board game Escape: The Curse of the Temple.
      </p>
    </>
  ),
  itch: new URL("https://lostimmortal.itch.io/hyperspace-roll"),
};
