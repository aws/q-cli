import logger from "loglevel";
import * as Sentry from "@sentry/react";
import { SETTINGS, getSetting } from "@amzn/fig-io-api-bindings-wrappers";

const getEnvironment = (): string => {
  switch (window.location.hostname) {
    case "autocomplete.fig.io":
    case "app.withfig.com":
      return "production";

    case "develop.autocomplete.fig.io":
    case "staging.autocomplete.fig.io":
    case "staging.withfig.com":
      return "staging";

    case "localhost":
      return "localhost";

    default:
      return "other";
  }
};

let didInitSentry = false;

const telemetryDisabled = () => getSetting(SETTINGS.TELEMETRY_DISABLED, false);

export const initSentry = async () => {
  if (!telemetryDisabled()) {
    Sentry.init({
      dsn: "https://baef329e0f534f92b53bf915b655c3b0@o436453.ingest.sentry.io/5829654",
      environment: getEnvironment(),
      integrations: [new Sentry.BrowserTracing()],
      tracesSampleRate: 1.0,
      release: __APP_VERSION__,
    });

    Sentry.setTag("pathname", window.location.pathname);

    didInitSentry = true;
  }
};

export const captureError = (err: Error, log = true) => {
  if (log) {
    logger.error(err);
  }
  if (!telemetryDisabled()) {
    if (!didInitSentry) {
      initSentry();
    }
    Sentry.captureException(err);
  }
};
