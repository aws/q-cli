import ModalContext from "@/context/modal";
import installChecks from "@/data/install";
import { InstallCheck } from "@/types/preferences";
import { Auth, Fig, Install, Internal, Native, State } from "@withfig/api-bindings";
import { useContext, useEffect, useState } from "react";
import { Button } from "../ui/button";
import { AwsLogo } from "../svg/icons";
import Lockup from "../svg/logo";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "../ui/collapsible";
import { Code } from "../text/code";
import onboarding from "@/data/onboarding";
import { ChevronDown } from "lucide-react";

export function WelcomeModal({ next }: { next: () => void }) {
  return (
    <div className="flex flex-col items-center gap-8 gradient-cw-secondary-light -m-10 p-4 pt-10 rounded-lg text-white">
      <div className="flex flex-col items-center gap-8">
        <Lockup />
        <div className="flex flex-col gap-2 items-center text-center">
        <h2 className="text-2xl text-white font-semibold select-none leading-none font-ember tracking-tight">
          Welcome!
        </h2>
        <p className="text-sm">Let's get your computer configured...</p>
        </div>
      </div>
      <div className="flex flex-col items-center gap-2 text-white text-sm font-bold">
        <Button
            variant="glass"
            onClick={() => next()}
            className="flex gap-4"
          >
            Get started
          </Button>
      </div>
    </div>
  )
}

export function LoginModal({ next }: { next: () => void }) {
  const [loginState, setLoginState] = useState<
    "not started" | "loading" | "logged in"
  >("not started");
  const [loginCode, setLoginCode] = useState<string | null>(null);

  async function handleLogin() {
    setLoginState("loading");

    const init = await Auth.builderIdStartDeviceAuthorization();
    setLoginCode(init.code);

    await Native.open(init.url);

    await Auth.builderIdPollCreateToken(init).catch(console.error);
    setLoginState("logged in");

    await Internal.sendWindowFocusRequest({});
  }

  useEffect(() => {
    if (loginState !== "logged in") return;

    Internal.sendOnboardingRequest({
      action: Fig.OnboardingAction.FINISH_ONBOARDING,
    });

    next();
  }, [loginState, next]);

  return (
    <div className="flex flex-col items-center gap-4 gradient-cw-secondary-light -m-10 p-4 pt-10 rounded-lg text-white">
      <div className="flex flex-col items-center gap-8">
        <Lockup />
        <h2 className="text-xl text-white font-semibold select-none leading-none font-ember tracking-tight">
          Sign in to get started
        </h2>
      </div>
      <div className="flex flex-col items-center gap-2 text-white text-sm font-bold">
        {loginCode ? (
          loginCode
        ) : (
          <Button
            variant="glass"
            onClick={() => handleLogin()}
            className="flex gap-4 pl-2"
          >
            <AwsLogo />
            Sign in
          </Button>
        )}
      </div>
    </div>
  );
}

export default function InstallModal() {
  const [step, setStep] = useState(0);
  const check = onboarding[step] as InstallCheck;
  const { setModal } = useContext(ModalContext);
  const [explainerOpen, setExplainerOpen] = useState(false);

  function handleInstall(key: InstallCheck["installKey"]) {
    if (!key) return;

    Install.install(key)
      .then(() => {
        if (step < installChecks.length - 1) {
          setStep(step + 1);
        } else {
          setModal(null);
        }
      })
      .catch((e) => {
        console.error(e);
        if (step < installChecks.length - 1) {
          setStep(step + 1);
        } else {
          setModal(null);
        }
      });
  }

  function handleFinish() {
    State.set('onboarding.completed', true)
    setModal(null);
  }

  if (check.id === 'welcome') {
    return <WelcomeModal next={() => setStep(step + 1)} />
  }

  if (check.id === "login") {
    return <LoginModal next={() => handleFinish()} />;
  }

  return (
    <div className="flex flex-col gap-4">
      <h2 className="font-medium text-lg select-none leading-none">
        {check.title}
      </h2>
      <div className="flex flex-col gap-2 text-base font-light text-zinc-500 select-none items-start leading-tight">
        {check.description.map((d, i) => (
          <p key={i} className="text-sm">{d}</p>
        ))}
        {check.image && (
          <img
            src={check.image}
            className="h-auto w-full min-h-40 rounded-sm bg-zinc-200 border border-zinc-300"
          />
        )}
      </div>
      <div className="flex flex-col gap-1">
        <Button onClick={() => handleInstall(check.installKey)}>
          {check.action}
        </Button>
        {check.explainer && (
          <Collapsible open={explainerOpen} onOpenChange={setExplainerOpen}>
            <CollapsibleTrigger asChild className="text-zinc-400">
              <div className="flex items-center">
              <ChevronDown
                className={`h-3 w-3 ${
                  explainerOpen ? "rotate-0" : "-rotate-90"
                } cursor-pointer text-zinc-400`}
              />
              <span className="text-xs text-zinc-400 select-none cursor-pointer">
                {check.explainer.title}
              </span>
              </div>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <ul className="flex flex-col gap-4 py-4">
                {check.explainer.steps.map((step, i) => {
                  return (
                    <li key={i} className="flex items-baseline gap-2 text-xs">
                      <span>{i + 1}.</span>
                      <p className="flex flex-wrap gap-[0.25em]">
                        {step.map((str, i) => {
                          switch (str.tag) {
                            case "code":
                              return <Code key={i}>{str.content}</Code>;
                            default:
                            case "span":
                              return <span key={i}>{str.content}</span>;
                          }
                        })}
                      </p>
                    </li>
                  );
                })}
              </ul>
            </CollapsibleContent>
          </Collapsible>
        )}
      </div>
    </div>
  );
}
