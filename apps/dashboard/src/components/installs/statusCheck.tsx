import { InstallCheck } from "@/types/preferences";
import { Install } from "@withfig/api-bindings";
import { useEffect, useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "../ui/collapsible";
import { Check, ChevronDown, X } from "lucide-react";
import { Button } from "../ui/button";

export default function StatusCheck({ check }: { check: InstallCheck }) {
  const [needsToBeChecked, setNeedsToBeChecked] = useState(false)
  const [status, setStatus] = useState(false);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    if ((status && !needsToBeChecked) || !check.installKey) return

    Install.isInstalled(check.installKey).then((r) => {
      if (r === false) {
        setStatus(false)
        setNeedsToBeChecked(false)
        setExpanded(true)
      }

      setStatus(r)
      setNeedsToBeChecked(!r)
      setExpanded(!r)
    });

  }, [check.installKey, needsToBeChecked, status]);

  function fixInstall() {
    if (!check.installKey) return
    Install.install(check.installKey).then(() => setNeedsToBeChecked(true)).catch((e) => console.error(e));
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