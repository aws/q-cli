import ModalContext from "@/context/modal";
import { InstallCheck } from "@/types/preferences";
import { Auth, Fig, Install, Internal, Native } from "@withfig/api-bindings";
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
import { useStatusCheck } from "@/hooks/store/useStatusCheck";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { Input } from "../ui/input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";

// TODO: this should be fetched from https://idetoolkits.amazonwebservices.com/endpoints.json
const REGIONS = {
  "af-south-1": {
    description: "Africa (Cape Town)",
  },
  "ap-east-1": {
    description: "Asia Pacific (Hong Kong)",
  },
  "ap-northeast-1": {
    description: "Asia Pacific (Tokyo)",
  },
  "ap-northeast-2": {
    description: "Asia Pacific (Seoul)",
  },
  "ap-northeast-3": {
    description: "Asia Pacific (Osaka)",
  },
  "ap-south-1": {
    description: "Asia Pacific (Mumbai)",
  },
  "ap-south-2": {
    description: "Asia Pacific (Hyderabad)",
  },
  "ap-southeast-1": {
    description: "Asia Pacific (Singapore)",
  },
  "ap-southeast-2": {
    description: "Asia Pacific (Sydney)",
  },
  "ap-southeast-3": {
    description: "Asia Pacific (Jakarta)",
  },
  "ap-southeast-4": {
    description: "Asia Pacific (Melbourne)",
  },
  "ca-central-1": {
    description: "Canada (Central)",
  },
  "eu-central-1": {
    description: "Europe (Frankfurt)",
  },
  "eu-central-2": {
    description: "Europe (Zurich)",
  },
  "eu-north-1": {
    description: "Europe (Stockholm)",
  },
  "eu-south-1": {
    description: "Europe (Milan)",
  },
  "eu-south-2": {
    description: "Europe (Spain)",
  },
  "eu-west-1": {
    description: "Europe (Ireland)",
  },
  "eu-west-2": {
    description: "Europe (London)",
  },
  "eu-west-3": {
    description: "Europe (Paris)",
  },
  "il-central-1": {
    description: "Israel (Tel Aviv)",
  },
  "me-central-1": {
    description: "Middle East (UAE)",
  },
  "me-south-1": {
    description: "Middle East (Bahrain)",
  },
  "sa-east-1": {
    description: "South America (Sao Paulo)",
  },
  "us-east-1": {
    description: "US East (N. Virginia)",
  },
  "us-east-2": {
    description: "US East (Ohio)",
  },
  "us-west-1": {
    description: "US West (N. California)",
  },
  "us-west-2": {
    description: "US West (Oregon)",
  },
} as const;

const DEFAULT_SSO_REGION = "us-east-1";

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
        <Button variant="glass" onClick={() => next()} className="flex gap-4">
          Get started
        </Button>
      </div>
    </div>
  );
}

export function LoginModal({ next }: { next: () => void }) {
  const [loginState, setLoginState] = useState<
    "not started" | "loading" | "logged in"
  >("not started");
  const [loginCode, setLoginCode] = useState<string | null>(null);

  const [startUrl, setStartUrl] = useState("");
  const [region, setRegion] = useState(DEFAULT_SSO_REGION);

  async function handleLogin(startUrl?: string, region?: string) {
    setLoginState("loading");
    const init = await Auth.builderIdStartDeviceAuthorization({
      startUrl,
      region,
    }).catch((err) => {
      setLoginState("not started");
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
        console.error(err);
      });
  }

  useEffect(() => {
    Auth.status().then((r) => {
      console.log("auth status", r);
      setLoginState(r.builderId ? "logged in" : "not started");
    });
  }, []);

  useEffect(() => {
    if (loginState !== "logged in") return;

    next();
  }, [loginState, next]);

  return (
    <div className="flex flex-col items-center gap-8 gradient-cw-secondary-light -m-10 p-4 pt-10 rounded-lg text-white">
      <div className="flex flex-col items-center gap-8">
        <Lockup />
        <h2 className="text-xl text-white font-semibold select-none leading-none font-ember tracking-tight">
          Sign in to get started
        </h2>
      </div>
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
          <>
            <Tabs defaultValue="personal">
              <TabsList>
                <TabsTrigger value="personal">
                  Personal (Builder ID)
                </TabsTrigger>
                <TabsTrigger value="team">
                  Team (IAM Identity Center)
                </TabsTrigger>
              </TabsList>
              <TabsContent value="personal">
                <div className="border rounded p-4 flex flex-col gap-4 bg-black/20">
                  <div className="flex flex-col gap-1">
                    <p className="font-bold text-lg">AWS Builder ID</p>
                    <p>
                      With AWS Builder ID, sign in for free without an AWS
                      account.
                    </p>
                  </div>
                  <Button
                    variant="glass"
                    onClick={() => handleLogin()}
                    className="flex gap-4 pl-2 self-center"
                  >
                    <AwsLogo />
                    Sign up or Sign in
                  </Button>
                </div>
              </TabsContent>
              <TabsContent value="team">
                <div className="border rounded p-4 flex flex-col bg-black/20 gap-4">
                  <div>
                    <p className="font-bold text-lg">IAM Identity Center</p>
                    <p>Successor to AWS Single Sign-on</p>
                  </div>
                  <div className="flex flex-col gap-1">
                    <p className="font-bold">Start URL</p>
                    <p>
                      URL for your organization, provided by an admin or help
                      desk.
                    </p>
                    <Input
                      value={startUrl}
                      onChange={(e) => setStartUrl(e.target.value)}
                      className="text-black"
                      type="url"
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <p className="font-bold">Region</p>
                    <p>AWS Region that hosts Identity directory</p>
                    <Select
                      onValueChange={(value) => setRegion(value)}
                      value={region}
                    >
                      <SelectTrigger className="w-full text-black">
                        <SelectValue placeholder="Theme" />
                      </SelectTrigger>
                      <SelectContent className="h-96">
                        {Object.entries(REGIONS).map(([key, value]) => (
                          <SelectItem key={key} value={key}>
                            <span className="font-mono mr-2">{key}</span>
                            <span className="text-xs text-neutral-600">
                              {value.description}
                            </span>
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <Button
                    variant="glass"
                    onClick={() => {
                      handleLogin(startUrl, region);
                    }}
                    className="flex gap-4 pl-2 self-center"
                  >
                    <AwsLogo />
                    Sign in
                  </Button>
                </div>
              </TabsContent>
            </Tabs>
          </>
        )}
      </div>
    </div>
  );
}

type installKey = "dotfiles" | "accessibility" | "inputMethod";

function InstallModal({
  check,
  skip,
  next,
}: {
  check: InstallCheck;
  skip: () => void;
  next: () => void;
}) {
  const [explainerOpen, setExplainerOpen] = useState(false);
  const [isInstalled] = useStatusCheck(check.installKey as installKey);
  const [timeElapsed, setTimeElapsed] = useState(false);
  const [checking, setChecking] = useState(false);

  useEffect(() => {
    if (timeElapsed) return;

    const timer = setTimeout(() => setTimeElapsed(true), 5000);
    return () => clearTimeout(timer);
  }, [timeElapsed]);

  useEffect(() => {
    if (!isInstalled) return;

    next();
  }, [isInstalled, next]);

  function handleInstall(key: InstallCheck["installKey"]) {
    if (!key) return;

    if (checking || check.id === "dotfiles") {
      next();
      return;
    }

    Install.install(key)
      .then(() => setChecking(true))
      .catch((e) => console.error(e));
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex justify-between items-baseline">
        <h2 className="font-medium text-lg select-none leading-none">
          {check.title}
        </h2>
        {timeElapsed && (
          <button className={"text-xs text-black/50"} onClick={skip}>
            skip
          </button>
        )}
      </div>
      <div className="flex flex-col gap-2 text-base font-light text-zinc-500 select-none items-start leading-tight">
        {check.description.map((d, i) => (
          <p key={i} className="text-sm">
            {d}
          </p>
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
          {checking ? "Continue" : check.action}
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

export default function OnboardingModal() {
  const [step, setStep] = useState(0);
  const check = onboarding[step] as InstallCheck;
  const { setModal } = useContext(ModalContext);
  const [dotfilesCheck, refreshDotfiles] = useStatusCheck("dotfiles");
  const [accessibilityCheck, refreshAccessibility] =
    useStatusCheck("accessibility");

  // these let us skip steps
  const [dotfiles, setDotfiles] = useState(dotfilesCheck);
  const [accessibility, setAccessibility] = useState(accessibilityCheck);
  const checksComplete = dotfiles && accessibility;

  // console.log({ id: check.id, checksComplete, dotfiles, accessibility })

  useEffect(() => {
    refreshAccessibility();
    refreshDotfiles();
  }, [refreshAccessibility, refreshDotfiles]);

  useEffect(() => {
    if (!checksComplete) return;
    Internal.sendOnboardingRequest({
      action: Fig.OnboardingAction.FINISH_ONBOARDING,
    });
    setModal(null);
  }, [checksComplete, setModal]);

  function nextStep() {
    if (step >= onboarding.length - 1) {
      Internal.sendOnboardingRequest({
        action: Fig.OnboardingAction.FINISH_ONBOARDING,
      });
      setModal(null);
      return;
    }

    setStep(step + 1);
  }

  function skipInstall() {
    if (!check.id) return;

    if (check.id === "dotfiles") {
      setDotfiles(true);
      setStep(step + 1);
    }

    if (check.id === "accessibility") {
      setAccessibility(true);
      setStep(step + 1);
    }
  }

  if (check.id === "dotfiles" || check.id === "accessibility") {
    return <InstallModal check={check} skip={skipInstall} next={nextStep} />;
  }

  if (check.id === "welcome") {
    return <WelcomeModal next={nextStep} />;
  }

  if (check.id === "login") {
    return <LoginModal next={nextStep} />;
  }

  return null;
}
