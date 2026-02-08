import { ContentBlock } from "@/components/contentBlock";
import { HeightRestricted, Header } from "../examples/[example]/styles";
import { Playground } from "./playground";
import { Viewport } from "next";

export const viewport: Viewport = { interactiveWidget: "resizes-content" };

export default function Page() {
  return (
    <HeightRestricted>
      <ContentBlock color="#9fa6db" margin={0}>
        <Header>Playground</Header>
      </ContentBlock>
      <Playground />
    </HeightRestricted>
  );
}
