import { ShowcaseGame, shuffle } from "../../../games";
import h1 from "./the-hat-chooses-the-wizard-0.png";
import h2 from "./the-hat-chooses-the-wizard-1.png";
import h3 from "./the-hat-chooses-the-wizard-2.png";
import h4 from "./the-hat-chooses-the-wizard-3.png";

const HatWizScreenshots = [h1, h2, h3, h4];

export const HatWiz: ShowcaseGame = {
  name: "The Hat Chooses the Wizard",
  developers: shuffle(["Corwin Kuiper", "Gwilym Inzani"]),
  screenshots: HatWizScreenshots,
  description: (
    <>
      <p>
        &lsquo;The Hat Chooses the Wizard&rsquo; is a 2D platformer. This game
        was developed as an entry for the GMTK game jam 2021, with the theme
        &ldquo;joined together&rdquo;. The entire game, except for the music,
        was produced in just 48 hours.
      </p>
      <p>
        In this game, you play as a wizard searching for his missing staff.
        However, the path to the staff is filled with dangerous obstacles and
        monsters. Luckily, you have a powerful magic hat that can be thrown and
        recalled, allowing you to fly towards it and reach otherwise
        inaccessible platforms.
      </p>
      <p>
        With this unique mechanic, you can explore the game&apos;s levels and
        defeat enemies. The game&apos;s simple but challenging gameplay will put
        your platforming skills to the test as you try to reach the end.
      </p>
      <p>
        The music is by Otto Halmén released under creative commons attribution
        3.0 and can be found here:{" "}
        <a href="https://opengameart.org/content/sylvan-waltz-standard-looped-version">
          opengameart.org/content/sylvan-waltz-standard-looped-version
        </a>
      </p>
    </>
  ),
  itch: new URL("https://lostimmortal.itch.io/the-hat-chooses-the-wizard"),
};
