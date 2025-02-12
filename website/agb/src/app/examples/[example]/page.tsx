import { Examples } from "@/roms/examples/examples";
import { slugify } from "@/sluggify";
import { ContentBlock } from "@/components/contentBlock";
import * as fs from "node:fs/promises";
import { BackToExampleLink } from "./styles";
import { Example } from "./example";
import styled from "styled-components";

export async function generateStaticParams() {
  return Examples.map((example) => ({
    example: slugify(example.example_name),
  }));
}

function getExample(sluggedExample: string) {
  const example = Examples.find(
    (x) => slugify(x.example_name) === sluggedExample
  );
  if (!example) {
    throw new Error(`cannot find example ${sluggedExample}`);
  }

  return example;
}

async function loadSourceCode(exampleName: string) {
  const source = await fs.readFile(`src/roms/examples/${exampleName}.rs`);

  return source.toString();
}

export default async function Page({
  params,
}: {
  params: Promise<{ example: string }>;
}) {
  const { example: exampleParam } = await params;
  const example = getExample(exampleParam);
  const source = await loadSourceCode(example.example_name);

  return (
    <>
      <ContentBlock color="#9fa6db">
        <h1>Example: {exampleParam}</h1>
        <BackToExampleLink href={`../examples#${exampleParam}`}>
          <strong>&lt;</strong> Back to examples
        </BackToExampleLink>
      </ContentBlock>
      <Example exampleSlug={exampleParam} sourceCode={source} />
      <ContentBlock color="#f5755e">
        <></>
      </ContentBlock>
    </>
  );
}
