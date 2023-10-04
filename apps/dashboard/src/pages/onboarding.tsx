import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import ModalContext from "@/context/modal";
import { Install } from "@withfig/api-bindings";
import { Check, ChevronDown, X } from "lucide-react";
import { useContext, useEffect, useState } from "react";

type InstallCheck = {
  id: string
  installKey: "dotfiles" | "accessibility" | "inputMethod"
  title: string
  description: string[]
  example?: string
  action: string
};

const installChecks: InstallCheck[] = [
  {
    id: "shellIntegrations",
    installKey: "dotfiles",
    title: "Shell integrations",
    description:
      ["Pick your favorite tools to configure them for use with CodeWhisperer"],
    example: '/asdf',
    action: "Install",
  },
  {
    id: "accessibility",
    installKey: "accessibility",
    title: "Accessibility settings",
    description:
      ["Fig uses this permission to position the Autocomplete window and insert text on your behalf.", "If enabling it isn't working, try toggling it off and on again or restarting Fig."],
      example: '/asdf',
    action: "Enable",
  },
  {
    id: "inputMethod",
    installKey: "inputMethod",
    title: "Input methods",
    description:
      ["Integrate CodeWhisperer with your local shell so we can run the necessary hooks."],
      example: '/asdf',
      action: "Enable",
  },
];

export function InstallModal() {
  const [step, setStep] = useState(0)
  const check = installChecks[step] as InstallCheck;
  const { setModal } = useContext(ModalContext)

  function handleInstall (key: InstallCheck['installKey']) {
    Install.install(key)
      .then(() => {
        console.log(`step ${step + 1} complete`)
        if (step < installChecks.length - 1) {
          setStep(step + 1)
        } else {
          setModal(null)
        }
      })
      .catch((e) => {
        console.error(e)
        if (step < installChecks.length - 1) {
          setStep(step + 1)
        } else {
          setModal(null)
        }
      })
  }

  return (
    <div className="flex flex-col gap-4">
      <h2 className="font-medium text-lg select-none leading-none">{check.title}</h2>
      <div className="flex flex-col gap-2 text-base font-light text-zinc-500 select-none items-start leading-tight">
        {check.description.map((d, i) => <p key={i}>{d}</p>)}
        {check.example && <img src={check.example} className="h-auto w-full min-h-40 rounded-sm bg-zinc-200 border border-zinc-300" />}
      </div>
      <Button onClick={() => handleInstall(check.installKey)}>
        {check.action}
      </Button>
    </div>
  );
}

function StatusCheck({ check }: { check: InstallCheck }) {
  const [status, setStatus] = useState(false);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    Install.isInstalled(check.installKey).then((r) => {
      setStatus(r);
      if (r === false) setExpanded(true);
    });
  }, [check.installKey]);

  function fixInstall() {
    Install.install(check.installKey).catch((e) => console.error(e));
  }

  return (
    <Collapsible
      className="flex gap-4 self-stretch"
      open={expanded}
      onOpenChange={setExpanded}
    >
      <CollapsibleTrigger asChild className="mt-5 flex-none">
        <ChevronDown
          className={`h-5 w-5 ${
            expanded ? "rotate-0" : "-rotate-90"
          } cursor-pointer text-zinc-400`}
        />
      </CollapsibleTrigger>
      <div className="flex flex-col border-b-[1px] border-zinc-200 py-4 flex-auto gap-1">
        <div className="flex gap-2 items-center">
          <h2 className="font-medium text-lg select-none">{check.title}</h2>
          {status ? (
            <Check className="h-5 w-5 text-green-600" />
          ) : (
            <X className="h-5 w-5 text-red-600" />
          )}
        </div>
        <CollapsibleContent className="flex flex-col gap-2 text-base font-light text-zinc-500 select-none items-start leading-tight">
          {check.description.map((d, i) => <p key={i}>{d}</p>)}
          <Button
            onClick={fixInstall}
            disabled={status}
            className="disabled:bg-zinc-400 h-auto py-2 px-6 mt-1"
          >
            {status ? "Enabled" : "Enable"}
          </Button>
        </CollapsibleContent>
      </div>
    </Collapsible>
  );
}

export default function Page() {
  const { setModal } = useContext(ModalContext);

  function startOnboarding() {
    setModal(<InstallModal />);
  }

  return (
    <div className="flex flex-col items-start">
      <div className="flex justify-between gap-4 self-stretch">
        <h1 className="text-3xl font-black select-none mb-2">
          Finish onboarding
        </h1>
        <Button
          variant="ghost"
          onClick={startOnboarding}
          className="disabled:bg-zinc-400 h-auto py-2 px-6 mt-1"
        >
          Open flow
        </Button>
      </div>
      {installChecks.map((check) => {
        return <StatusCheck check={check} key={check.id} />;
      })}
    </div>
  );
}
