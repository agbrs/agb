import { Examples } from "@/roms/examples/examples";
import { slugify } from "@/sluggify";
import { Emulator } from "./emulator";
import { ContentBlock } from "@/components/contentBlock";

export async function generateStaticParams() {
  return Examples.map((example) => ({
    example: slugify(example.example_name),
  }));
}

export default function Page({ params }: { params: { example: string } }) {
  return (
    <>
      <ContentBlock color="#9fa6db">
        <h1>Example: {params.example}</h1>
      </ContentBlock>
      <ContentBlock>
        <Emulator exampleName={params.example} />
      </ContentBlock>
      <ContentBlock color="#f5755e">
        <></>
      </ContentBlock>
    </>
  );
}
