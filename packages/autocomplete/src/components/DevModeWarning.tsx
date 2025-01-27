import React from "react";

function DevModeWarning({
  devMode,
  suggestionWidth,
}: {
  devMode: unknown;
  suggestionWidth: number;
}): React.JSX.Element {
  return (
    <>
      {Boolean(devMode) && (
        <div
          style={{
            width: suggestionWidth - 20,
          }}
          className="m-1 space-y-1.5 rounded bg-amber-500 px-2.5 py-2 text-black"
        >
          <div className="text-base font-bold">Developer mode enabled!</div>
          <div className="text-sm">
            Loading specs from disk. Disable with either
          </div>
          <div className="ml-2 flex flex-col gap-1 text-xs">
            <div>
              •{" "}
              <code className="rounded-sm bg-zinc-700 p-0.5 text-zinc-200">
                Ctrl + C
              </code>{" "}
              in the dev mode process
            </div>
            <div>
              {"• "}
              <button
                type="button"
                className="text-xs underline"
                onClick={() => {}}
              >
                Click to disable
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

export default DevModeWarning;
