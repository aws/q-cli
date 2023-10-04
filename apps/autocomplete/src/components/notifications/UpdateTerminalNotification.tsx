import { ShellContext } from "@fig/fig-api-proto/dist/fig_common.pb";
import { useAutocompleteStore } from "../../state";
import { Notification } from "./Notification";

export const UpdateTerminalNotification = () => {
  const { figState } = useAutocompleteStore();
  const shellContext = figState.shellContext as ShellContext | undefined;

  const figtermVersion = shellContext?.figtermVersion;
  const desktopVersion = window.fig.constants?.version;

  const misMatchedVersions = figtermVersion !== desktopVersion;

  const isLegacyVersion = parseInt((fig.constants?.version ?? "0")[0], 10) < 2;

  return (
    <Notification
      localStorageKey={`update_terminal_${
        shellContext?.sessionId ?? "unknown"
      }`}
      show={
        Boolean(figtermVersion) &&
        Boolean(desktopVersion) &&
        misMatchedVersions &&
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
          an older version of Fig, please restart it.
        </>
      }
    />
  );
};
