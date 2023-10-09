import InstallModal from "@/components/installs/modal";
import StatusCheck from "@/components/installs/statusCheck";
import { Button } from "@/components/ui/button";
import ModalContext from "@/context/modal";
import installChecks from "@/data/install";
import { useContext } from "react";

export default function Page() {
  const { setModal } = useContext(ModalContext);

  function startOnboarding() {
    setModal(<InstallModal />);
  }

  return (
    <div className="flex flex-col items-start">
      <div className="flex justify-between gap-4 self-stretch">
        <h1 className="text-3xl font-black select-none mb-2">
          Getting started
        </h1>
        <Button
          variant="ghost"
          onClick={startOnboarding}
          className="disabled:bg-zinc-400 h-auto py-2 px-6 mt-1"
        >
          Open flow
        </Button>
      </div>
      {/* {installChecks.map((check) => {
        return <StatusCheck check={check} key={check.id} />;
      })} */}
    </div>
  );
}
