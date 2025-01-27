import React, { forwardRef } from "react";

interface AutocompleteWindowProps {
  children: React.ReactNode;
}

const AutocompleteWindow = forwardRef<HTMLDivElement, AutocompleteWindowProps>(
  (props, ref) => (
    <div
      id="autocompleteWindow"
      className="q-autocomplete-wrapper relative flex flex-col overflow-hidden"
      ref={ref}
    >
      {props.children}
    </div>
  ),
);
AutocompleteWindow.displayName = "AutocompleteWindow";

export default AutocompleteWindow;
