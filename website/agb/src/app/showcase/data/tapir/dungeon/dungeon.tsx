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
        You are the puzzle designer for a dungeon. No, you prefer to think of
        yourself as a visionary architect. A fiendish master of traps and
        puzzles designed to foil any attempts from so-called heroes to reach the
        treasure.
      </p>
      <p>
        However, something changed recently. Heroes keep moving the same way, in
        predictable patterns. They aren't trying to solve your puzzles any more,
        or fight the monsters you've carefully placed to ambush them!
      </p>
      <p>
        Looks like you'll have to apply your unique talents in other ways. Get
        the hero through the dungeon and maybe they'll snap out of it?
      </p>
    </>
  ),
  itch: new URL("https://setsquare.itch.io/dungeon-puzzlers-lament"),
};
