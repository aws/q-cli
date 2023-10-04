import { createErrorInstance } from "@internal/shared/errors";

export const MissingSpecError = createErrorInstance("MissingSpecError");
export const HistoryReadingError = createErrorInstance("HistoryReadingError");
export const SuggestionNotFoundError = createErrorInstance(
  "SuggestionNotFoundError",
);
