export function slugify(x: string) {
  return x
    .toLowerCase()
    .split(" ")
    .join("-")
    .replaceAll(/[^a-zA-Z0-9\-_]/g, "");
}
