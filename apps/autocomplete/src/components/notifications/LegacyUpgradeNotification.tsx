import { useLoggedIn } from "../../hooks/useLoggedIn";
import { useAutocompleteStore } from "../../state";
import { Notification } from "./Notification";

export const LegacyUpgradeNotification = () => {
  const { settings } = useAutocompleteStore();
  const isLoggedIn = useLoggedIn();

  const isBetaUser =
    isLoggedIn && (settings as Record<string, unknown>)["app.beta"] === true;
  const isLegacyVersion = parseInt((fig.constants?.version ?? "0")[0], 10) < 2;

  return (
    <Notification
      localStorageKey="legacy_upgrade_mac"
      show={isLegacyVersion && isBetaUser}
      title={
        <>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            aria-hidden="true"
            className="mb-0.5 mr-1 h-5 w-5"
          >
            <path
              d="M13.75
              7h-3v5.296l1.943-2.048a.75.75
              0
              011.114
              1.004l-3.25
              3.5a.75.75
              0
              01-1.114
              0l-3.25-3.5a.75.75
              0
              111.114-1.004l1.943
              2.048V7h1.5V1.75a.75.75
              0
              00-1.5
              0V7h-3A2.25
              2.25
              0
              004
              9.25v7.5A2.25
              2.25
              0
              006.25
              19h7.5A2.25
              2.25
              0
              0016
              16.75v-7.5A2.25
              2.25
              0
              0013.75 7z"
            />
          </svg>
          <span>There is an update available!</span>
        </>
      }
      description={<>Click &quot;Check for Updates&quot; in the menu.</>}
    />
  );
};
