import { StaticImageData } from "next/image";
import { ReactNode } from "react";
import { HatWiz } from "./data/tapir/hatwiz/hatwiz";
import { Purple } from "./data/tapir/purple/purple";
import { Hyperspace } from "./data/tapir/hyperspace/hyperspace";
import { Dungeon } from "./data/tapir/dungeon/dungeon";

export interface ShowcaseGame {
  name: string;
  developers: string[];
  rom?: URL;
  screenshots: StaticImageData[];
  description: ReactNode;
  itch?: URL;
  otherLink?: URL;
}

export function shuffle<T>(a: T[]) {
  var j, x, i;
  for (i = a.length - 1; i > 0; i--) {
    j = Math.floor(Math.random() * (i + 1));
    x = a[i];
    a[i] = a[j];
    a[j] = x;
  }
  return a;
}

export const Games: ShowcaseGame[] = shuffle([
  HatWiz,
  Purple,
  Hyperspace,
  Dungeon,
]);
