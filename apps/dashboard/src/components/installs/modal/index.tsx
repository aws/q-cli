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
import { useLocalState } from "@/hooks/store/useState";
import migrate_dark from "@assets/images/fig-migration/dark.png?url";

export function WelcomeModal({ next }: { next: () => void }) {
  return (
    <div className="flex flex-col items-center gap-8 gradient-q-secondary-light -m-10 p-4 pt-10 rounded-lg text-white">
      <div className="flex flex-col items-center gap-8">
        <Lockup />
        <div className="flex flex-col gap-2 items-center text-center">
          <h2 className="text-2xl text-white font-semibold select-none leading-none font-ember tracking-tight">
            Welcome!
          </h2>
          <p className="text-sm">Let's get you set up...</p>
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
  const [migrationStarted] = useLocalState("desktop.migratedFromFig");
  const [migrationEnded, setMigrationEnded] = useLocalState(
    "desktop.migratedFromFig.UiComplete",
  );
  const { setModal } = useContext(ModalContext);
  const [dotfilesCheck, refreshDotfiles] = useStatusCheck("dotfiles");
  const [accessibilityCheck, refreshAccessibility] =
    useStatusCheck("accessibility");

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_dotfiles, setDotfiles] = useState(dotfilesCheck);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_accessibility, setAccessibility] = useState(accessibilityCheck);

  const isMigrating =
    Boolean(migrationStarted) === true && Boolean(migrationEnded) === false;

  useEffect(() => {
    refreshAccessibility();
    refreshDotfiles();
  }, [refreshAccessibility, refreshDotfiles]);

  function nextStep() {
    if (migrationStarted && !migrationEnded) {
      setMigrationEnded(true);
    }

    setStep(step + 1);
  }

  function finish() {
    refreshAccessibility();
    refreshDotfiles();
    Internal.sendOnboardingRequest({
      action: Fig.OnboardingAction.FINISH_ONBOARDING,
    });
    setModal(null);
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

  if (check.id === "welcome" && isMigrating) {
    return <FigMigrationModal next={nextStep} />;
  }

  if (check.id === "welcome") {
    return <WelcomeModal next={nextStep} />;
  }

  if (check.id === "dotfiles" || check.id === "accessibility") {
    return <InstallModal check={check} skip={skipInstall} next={nextStep} />;
  }

  if (check.id === "login") {
    return <LoginModal next={finish} />;
  }

  return null;
}

export function FigMigrationModal({ next }: { next: () => void }) {
  return (
    <div className="flex flex-col items-center gap-8 gradient-q-secondary-light -m-10 p-4 pt-10 rounded-lg text-white">
      <div className="flex flex-col items-center gap-8">
        <img src={migrate_dark} className="w-40" />
        <div className="flex flex-col gap-2 items-center text-center">
          <h2 className="text-2xl text-white font-semibold select-none leading-none font-ember tracking-tight">
            Almost done upgrading!
          </h2>
          <p className="text-sm">Let's get you set up...</p>
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
