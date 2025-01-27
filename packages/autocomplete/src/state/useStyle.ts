import { StyleType } from "./types";
import { useAutocomplete } from "./useAutocomplete";

export function useStyleType(): StyleType {
  return useAutocomplete((state) => state.styleType);
}

export function useClassname(className: string, tailwindClassName: string) {
  const styleType = useStyleType();
  switch (styleType) {
    case "tailwind":
      return tailwindClassName;
    case "class":
      return className;
    default:
      return className;
  }
}
