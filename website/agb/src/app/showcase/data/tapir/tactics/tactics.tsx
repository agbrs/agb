import { ShowcaseGame, shuffle } from "@/app/showcase/games";
import d1 from "./dungeon-tactics-advance-0.png";
import d2 from "./dungeon-tactics-advance-1.png";

const Screenshots = [d2, d1];

export const Tactics: ShowcaseGame = {
  name: "Dungeon Tactics Advance",
  developers: shuffle([
    "Corwin Kuiper",
    "Gwilym Inzani",
    "Sam Williams",
    "Ján Letovanec",
  ]),
  screenshots: Screenshots,
  description: (
    <>
      <p>
        In this rogue-lite turn based strategy game, you are tasked with
        reaching the depths of the dungeon and defeating what lies waiting for
        curious adventurers.
      </p>
    </>
  ),
  itch: new URL("https://setsquare.itch.io/dungeon-tactics-advance"),
};
