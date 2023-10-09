import ModalContext from "@/context/modal";
import installChecks from "@/data/install";
import { InstallCheck } from "@/types/preferences";
import { Auth, Install, Internal, Native } from "@withfig/api-bindings";
import { useContext, useEffect, useState } from "react";
import { Button } from "../ui/button";
import { AwsLogo } from "../svg/icons";
import Lockup from "../svg/logo";

function LoginModal({ next }: { next: () => void}) {
  const [loginState, setLoginState] = useState<'not started' | 'loading' | 'logged in'>('not started')
  const [loginCode, setLoginCode] = useState<string | null>(null);
  console.log({ loginState })

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
    if (loginState !== 'logged in') return

    next()
  }, [loginState, next])

  return (
    <div className="flex flex-col items-center gap-4 gradient-cw-secondary-light -m-4 p-4 pt-8 rounded-lg text-white">
      <div className="flex flex-col items-center gap-8">
      <Lockup />
      <h2 className="text-xl text-white font-semibold select-none leading-none font-ember tracking-tight">Sign in to get started</h2>
      </div>
      <div className="flex flex-col items-center gap-2 text-white text-sm font-bold">
        {loginCode 
          ? loginCode 
          : <Button variant='glass' onClick={() => handleLogin()} className="flex gap-4 pl-2">
            <AwsLogo />
            Sign in
            </Button>
        }
      </div>
    </div>
  )
}

export default function InstallModal() {
  const [step, setStep] = useState(0)
  const check = installChecks[step] as InstallCheck;
  const { setModal } = useContext(ModalContext)

  function handleInstall (key: InstallCheck['installKey']) {
    if (!key) return

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

  function handleFinish() {
    setModal(null)
  }

  if (check.id === 'login') {return <LoginModal next={() => handleFinish()} />}

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