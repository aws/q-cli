export function alphaByTitle(a: { title: string }, b: { title: string }) {
  if (a.title > b.title) return 1
  if (a.title < b.title) return -1

  return 0
}