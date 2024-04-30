import { ShowcaseGame, shuffle } from "@/app/showcase/games";
import d1 from "./the-dungeon-puzzlers-lament-0.png";
import d2 from "./the-dungeon-puzzlers-lament-1.png";

const Screenshots = [d1, d2];

export const Dungeon: ShowcaseGame = {
  name: "The Dungeon Puzzler's Lament",
  developers: shuffle(["Corwin Kuiper", "Gwilym Inzani"]),
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
  itch: new URL("https://setsquare.itch.io/dungeon-puzzlers-lament"),
};
