import { UserPrefView } from "@/components/preference/list";
import settings from "@/data/integrations";
import { parseBackticksToCode } from "@/lib/strings";

export default function Page() {
  const setupString =
    "Just run `cw integrations install input-method` then restart your computer to try it out.";

  return (
    <>
      <UserPrefView array={settings} />
      <div className="flex flex-col p-4 gap-1 rounded-lg bg-zinc-50 dark:bg-zinc-900 border border-zinc-100 dark:border-zinc-700">
        <h2 className="font-bold font-ember text-lg items-center flex">
          <span className="uppercase py-1 px-2 bg-cyan-500 font-mono text-white text-xs mr-2 rounded-sm">
            Beta
          </span>
          <span>Want support for JetBrains, Alacritty, and Kitty?</span>
        </h2>
        <p>{parseBackticksToCode(setupString)}</p>
      </div>
    </>
  );
}
