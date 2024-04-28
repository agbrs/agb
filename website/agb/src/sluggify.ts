export function slugify(x: string) {
  return x.toLowerCase().split(" ").join("-");
}
