import { ShowcaseGame, shuffle } from "@/app/showcase/games";
import k1 from "./khieras-quest-0.png";
import k2 from "./khieras-quest-1.png";
import k3 from "./khieras-quest-2.png";

const Screenshots = [k2, k3, k1];

export const Khiera: ShowcaseGame = {
  name: "Khiera's Quest",
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
        Khiera&apos;s quest is a platforming game about where the direction of
        gravity isn&apos;t always fixed. You&apos;ll find yourself jumping
        between planets, asteroids and other strange satellites to collect
        power-ups which will let you progress further. You will need to
        backtrack to previous locations to complete your quest.
      </p>
    </>
  ),
  itch: new URL("https://setsquare.itch.io/khieras-quest"),
};
