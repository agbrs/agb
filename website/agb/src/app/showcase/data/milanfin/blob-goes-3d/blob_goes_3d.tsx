import { ShowcaseGame } from "@/app/showcase/games";
import b0 from "./blob_goes_3d-0.png";
import b1 from "./blob_goes_3d-1.png";
import b2 from "./blob_goes_3d-2.png";
import b3 from "./blob_goes_3d-3.jpg";

const Screenshots = [b3, b0, b1, b2];

export const BlobGoes3d: ShowcaseGame = {
  name: "Blob Goes 3D",
  developers: ["MilanFIN"],
  screenshots: Screenshots,
  description: (
    <>
      <p>A 3d platformer for the Game Boy Advance.</p>
      <p>
        Source code is available{" "}
        <a
          href="https://github.com/MilanFIN/blob-goes-3d"
          target="_blank"
          rel="noopener"
        >
          on GitHub
        </a>
        . The repo includes instructions on how to create and bundle new levels
        into the rom file. If you want your own levels included in the game,
        feel free to make a pull request.
      </p>
    </>
  ),
  itch: new URL("https://milanfin.itch.io/blob-goes-3d"),
};
