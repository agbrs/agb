import { ShowcaseGame, shuffle } from "@/app/showcase/games";
import p1 from "./the-purple-night-0.png";
import p2 from "./the-purple-night-1.png";

const Screenshots = [p1, p2];

export const Purple: ShowcaseGame = {
  name: "The Purple Night",
  developers: shuffle(["Corwin Kuiper", "Gwilym Inzani", "Sam Williams"]),
  screenshots: Screenshots,
  description: (
    <>
      <p>Save a lost soul and take them safely back to the afterlife!</p>
      <p>
        The purple night is a platformer game where your health bar is your
        sword. The more damage you take, the shorter your sword gets, making you
        more nimble and your attacks faster, but also increasing your risk.
      </p>
      <p>
        Do you choose to stay at high health but low mobility, or low health and
        higher mobility?
      </p>
    </>
  ),
  itch: new URL("https://lostimmortal.itch.io/the-purple-night"),
};
