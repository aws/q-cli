import { ShellContext } from "@fig/fig-api-proto/dist/fig_common.pb";
import { useAutocompleteStore } from "../../state";
import { Notification } from "./Notification";

export const UpdateTerminalNotification = () => {
  const { figState } = useAutocompleteStore();
  const shellContext = figState.shellContext as ShellContext | undefined;

  const cwtermVersion = shellContext?.cwtermVersion;
  const desktopVersion = window.fig.constants?.version;

  const mismatchedVersions = cwtermVersion !== desktopVersion;

  const isLegacyVersion = parseInt((fig.constants?.version ?? "0")[0], 10) < 2;

  return (
    <Notification
      localStorageKey={`update_terminal_${
        shellContext?.sessionId ?? "unknown"
      }`}
      show={
        Boolean(cwtermVersion) &&
        Boolean(desktopVersion) &&
        mismatchedVersions &&
        !isLegacyVersion
      }
      title={
        <>
          <span>This terminal must be restarted</span>
        </>
      }
      description={
        <>
          This terminal is running integrations from <br />
          an older version of CodeWhisperer, please restart it.
        </>
      }
    />
  );
};
