import { useAutocomplete } from ".";

export function useClassName(
  className?: string,
  tailwind?: string,
): string | undefined {
  const { styleType } = useAutocomplete();
  switch (styleType) {
    case "class":
      return className;
    case "tailwind":
      return tailwind;
    default:
      return undefined;
  }
}
