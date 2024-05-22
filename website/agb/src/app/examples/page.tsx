import { Metadata } from "next";
import { ContentBlock } from "@/components/contentBlock";
import { slugify } from "@/sluggify";
import { GameDisplay, GameGrid, GameImage } from "./styles";
import { Examples } from "@/roms/examples/examples";

export const metadata: Metadata = {
  title: "Examples - agb",
};

export default function ShowcasePage() {
  return (
    <>
      <ContentBlock color="#9fa6db">
        <h1>Examples</h1>
      </ContentBlock>
      <ContentBlock uncentered>
        <GameGrid>
          {Examples.map((example, idx) => (
            <Game key={idx} example={example} />
          ))}
        </GameGrid>
      </ContentBlock>
    </>
  );
}

function Game({ example }: { example: (typeof Examples)[number] }) {
  const screenshot = example.screenshot;
  return (
    <GameDisplay
      href={`./examples/${slugify(example.example_name)}`}
      id={slugify(example.example_name)}
    >
      <GameImage
        src={screenshot}
        alt={`Screenshot of ${example.example_name}`}
      />
      <h2>{example.example_name}</h2>
    </GameDisplay>
  );
}
