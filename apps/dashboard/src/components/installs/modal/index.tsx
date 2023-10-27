import ModalContext from "@/context/modal";
import { InstallCheck } from "@/types/preferences";
import { Fig, Internal } from "@withfig/api-bindings";
import { useContext, useEffect, useState } from "react";
import { Button } from "../../ui/button";
import Lockup from "../../svg/logo";
import onboarding from "@/data/onboarding";
import { useStatusCheck } from "@/hooks/store/useStatusCheck";
import LoginModal from "./login";
import InstallModal from "./install";


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
