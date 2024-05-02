import Lockup from "@/components/svg/logo";
import { Button } from "@/components/ui/button";
import { Auth, Internal, Native } from "@withfig/api-bindings";
import { useEffect, useState } from "react";
import Tab from "./tabs";
import { useLocalStateZodDefault } from "@/hooks/store/useState";
import { z } from "zod";
import { Link } from "@/components/ui/link";
import { Q_MIGRATION_URL } from "@/lib/constants";

export default function LoginModal({ next }: { next: () => void }) {
  const [loginState, setLoginState] = useState<
    "not started" | "loading" | "logged in"
  >("not started");
  const [loginCode, setLoginCode] = useState<string | null>(null);
  const [tab, setTab] = useState<"builderId" | "iam">("builderId");
  const [error, setError] = useState<string | null>(null);
  const [completedOnboarding] = useLocalStateZodDefault(
    "desktop.completedOnboarding",
    z.boolean(),
    false,
  );

  async function handleLogin(startUrl?: string, region?: string) {
    setLoginState("loading");
    const init = await Auth.builderIdStartDeviceAuthorization({
      startUrl,
      region,
    }).catch((err) => {
      setLoginState("not started");
      setLoginCode(null);
      setError(err.message);
      console.error(err);
    });

    if (!init) return;

    setLoginCode(init.code);

    Native.open(init.url).catch((err) => {
      console.error(err);
    });

    await Auth.builderIdPollCreateToken(init)
      .then(() => {
        setLoginState("logged in");
        Internal.sendWindowFocusRequest({});
        next();
      })
      .catch((err) => {
        setLoginState("not started");
        setLoginCode(null);
        setError(err.message);
        console.error(err);
      });
  }

  useEffect(() => {
    Auth.status().then((r) => {
      setLoginState(r.authed ? "logged in" : "not started");
    });
  }, []);

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
        {loginCode ? (
          <>
            <p className="text-center w-80">
              Confirm code <span className="font-bold">{loginCode}</span> in the
              login page opened in your web browser.
            </p>
            <Button
              variant="glass"
              className="self-center w-32"
              onClick={() => {
                setLoginState("not started");
                setLoginCode(null);
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
