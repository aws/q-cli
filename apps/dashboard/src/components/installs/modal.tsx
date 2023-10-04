import ModalContext from "@/context/modal";
import installChecks from "@/data/install";
import { InstallCheck } from "@/types/preferences";
import { Install } from "@withfig/api-bindings";
import { useContext, useState } from "react";
import { Button } from "../ui/button";

export default function InstallModal() {
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