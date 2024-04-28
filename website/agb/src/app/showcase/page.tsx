import { Metadata } from "next";
import { ContentBlock } from "@/components/contentBlock";
import { Games, ShowcaseGame } from "./games";
import Link from "next/link";
import { slugify } from "@/sluggify";
import { GameDisplay, GameGrid, GameImage } from "./styles";
import Image from "next/image";

export const metadata: Metadata = {
  title: "Games made with agb",
};

export default function ColourPickerPage() {
  return (
    <>
      <ContentBlock color="#AAAFFF">
        <h1>Showcase</h1>
      </ContentBlock>
      <ContentBlock uncentered>
        <GameGrid>
          {Games.map((game, idx) => (
            <Game key={idx} game={game} />
          ))}
        </GameGrid>
      </ContentBlock>
    </>
  );
}

function Game({ game }: { game: ShowcaseGame }) {
  const lastImage = game.screenshots[game.screenshots.length - 1];
  return (
    <GameDisplay>
      <Link href={`./showcase/${slugify(game.name)}`}>
        <GameImage>
          <Image src={lastImage} alt={`Screenshot of ${game.name}`} />
        </GameImage>
        <h2>{game.name}</h2>
      </Link>
    </GameDisplay>
  );
}
