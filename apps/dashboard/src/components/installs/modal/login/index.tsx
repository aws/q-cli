import Lockup from "@/components/svg/logo";
import { Button } from "@/components/ui/button";
import { Auth, Internal, Native } from "@amzn/fig-io-api-bindings";
import { useEffect, useState } from "react";
import Tab from "./tabs";
import { useLocalStateZodDefault } from "@/hooks/store/useState";
import { z } from "zod";
import { Link } from "@/components/ui/link";
import { Q_MIGRATION_URL } from "@/lib/constants";
import { useAuth, useAuthRequest, useRefreshAuth } from "@/hooks/store/useAuth";

export default function LoginModal({ next }: { next: () => void }) {
  const midway = window?.fig?.constants?.midway ?? false;

  const [loginState, setLoginState] = useState<
    "not started" | "loading" | "logged in"
  >("not started");
  const [tab, setTab] = useLocalStateZodDefault<"builderId" | "iam">(
    "dashboard.loginTab",
    z.enum(["builderId", "iam"]),
    midway ? "iam" : "builderId",
  );
  const [currAuthRequestId, setAuthRequestId] = useAuthRequest();

  // console.log(tab);

  const [error, setError] = useState<string | null>(null);
  const [completedOnboarding] = useLocalStateZodDefault(
    "desktop.completedOnboarding",
    z.boolean(),
    false,
  );
  const auth = useAuth();
  const refreshAuth = useRefreshAuth();

  async function handleLogin(issuerUrl?: string, region?: string) {
    setLoginState("loading");
    setError(null);
    // We need to reset the auth request state before attempting, otherwise
    // an expected auth request cancellation will be presented to the user
    // as an error.
    setAuthRequestId(undefined);
    const init = await Auth.startPkceAuthorization({
      issuerUrl,
      region,
    }).catch((err) => {
      setLoginState("not started");
      setError(err.message);
      console.error(err);
    });

    if (!init) return;
    setAuthRequestId(init.authRequestId);

    Native.open(init.url).catch((err) => {
      console.error(err);
    });

    await Auth.finishPkceAuthorization(init)
      .then(() => {
        setLoginState("logged in");
        Internal.sendWindowFocusRequest({});
        refreshAuth();
        next();
      })
      .catch((err) => {
        // If this promise was originally for some older request attempt,
        // then we should just ignore the error.
        if (currAuthRequestId() === init.authRequestId) {
          setLoginState("not started");
          setError(err.message);
          console.error(err);
        }
      });
  }

  useEffect(() => {
    setLoginState(auth.authed ? "logged in" : "not started");
  }, [auth]);

  useEffect(() => {
    if (loginState !== "logged in") return;
    next();
  }, [loginState, next]);

  return (
    <div className="flex flex-col items-center gap-8 gradient-q-secondary-light -m-10 pt-10 p-4 rounded-lg text-white">
      <div className="flex flex-col items-center gap-8">
        <Lockup />
        {!completedOnboarding && (
          <h2 className="text-xl text-white font-semibold select-none leading-none font-ember tracking-tight">
            Sign in to get started
          </h2>
        )}
        {completedOnboarding && tab == "builderId" && (
          <div className="text-center flex flex-col">
            <div className="font-ember font-bold">
              CodeWhisperer is now Amazon Q
            </div>
            <Link href={Q_MIGRATION_URL} className="text-sm">
              Read the announcement blog post
            </Link>
          </div>
        )}
      </div>
      {error && (
        <div className="w-full bg-red-200 border border-red-600 rounded py-1 px-1">
          <p className="text-black dark:text-white font-semibold text-center">
            Failed to login
          </p>
          <p className="text-black dark:text-white text-center">{error}</p>
        </div>
      )}
      <div className="flex flex-col gap-4 text-white text-sm">
        {loginState === "loading" ? (
          <>
            <p className="text-center w-80">
              Waiting for authentication in the browser to complete...
            </p>
            <Button
              variant="glass"
              className="self-center w-32"
              onClick={() => {
                setLoginState("not started");
              }}
            >
              Back
            </Button>
          </>
        ) : (
          <Tab
            tab={tab}
            handleLogin={handleLogin}
            toggleTab={
              tab === "builderId"
                ? () => setTab("iam")
                : () => setTab("builderId")
            }
            signInText={completedOnboarding ? "Log back in" : "Sign in"}
          />
        )}
      </div>
    </div>
  );
}
