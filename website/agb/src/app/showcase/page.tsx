import { Metadata } from "next";
import { ContentBlock } from "@/components/contentBlock";
import { Games, ShowcaseGame } from "./games";
import { slugify } from "@/sluggify";
import { GameDisplay, GameGrid, GameImage } from "./styles";

export const metadata: Metadata = {
  title: "Showcase - agb",
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
  const showcaseImage = game.screenshots[game.screenshots.length - 1];
  return (
    <GameDisplay href={`./showcase/${slugify(game.name)}`}>
      <GameImage src={showcaseImage} alt={`Screenshot of ${game.name}`} />
      <h2>{game.name}</h2>
    </GameDisplay>
  );
}
