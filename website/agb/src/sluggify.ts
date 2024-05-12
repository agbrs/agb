export function slugify(x: string) {
  return x
    .toLowerCase()
    .split(" ")
    .join("-")
    .replace(/[^a-zA-Z0-9\-]/, "");
}
