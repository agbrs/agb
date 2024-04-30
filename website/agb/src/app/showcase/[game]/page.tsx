import { slugify } from "@/sluggify";
import { Games, ShowcaseGame } from "../games";
import { ContentBlock } from "@/components/contentBlock";
import { ExternalLink, ExternalLinkBlock } from "@/components/externalLink";
import Link from "next/link";
import {
  BackToShowcaseWrapper,
  DescriptionAndScreenshots,
  Description,
  Screenshots,
} from "./styles";

export async function generateStaticParams() {
  return Games.map((game) => ({
    game: slugify(game.name),
  }));
}

function getGame(slug: string) {
  const game = Games.find((game) => slugify(game.name) === slug);
  if (!game) {
    throw new Error("Not valid game name, this should never happen");
  }

  return game;
}

export function generateMetadata({ params }: { params: { game: string } }) {
  const game = getGame(params.game);
  return { title: game.name };
}

export default function Page({ params }: { params: { game: string } }) {
  const game = getGame(params.game);
  return <Display game={game} />;
}

function DeveloperNames({ names }: { names: string[] }) {
  if (names.length === 0) {
    throw new Error("You must specify developer names");
  }
  if (names.length === 1) {
    return names[0];
  }
  if (names.length === 2) {
    return names.join(" and ");
  }
  const first = names.slice(0, -1);
  return first.join(", ") + `, and ${names[names.length - 1]}`;
}

function Display({ game }: { game: ShowcaseGame }) {
  return (
    <>
      <ContentBlock color="#AAAFFF">
        <BackToShowcaseWrapper>
          <Link href={`../showcase#${slugify(game.name)}`}>
            <strong>&lt;</strong> Back to showcase
          </Link>
        </BackToShowcaseWrapper>
        <h1>{game.name}</h1>
        <div>
          By: <DeveloperNames names={game.developers} />
        </div>
      </ContentBlock>
      <ContentBlock>
        <DescriptionAndScreenshots>
          <Description>{game.description}</Description>
          <Screenshots screenshots={game.screenshots} />
        </DescriptionAndScreenshots>
      </ContentBlock>
      <ContentBlock color="#f5755e">
        <ExternalLinkBlock>
          {game.itch && (
            <ExternalLink href={game.itch.href}>View on itch.io</ExternalLink>
          )}
        </ExternalLinkBlock>
      </ContentBlock>
    </>
  );
}
