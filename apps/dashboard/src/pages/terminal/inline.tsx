import { UserPrefView } from "@/components/preference/list";
import { Code } from "@/components/text/code";
import { Kbd } from "@/components/ui/keystrokeInput";
import { Terminal } from "@/components/ui/terminal";
import settings, { intro } from "@/data/inline";
import inlineDemo from "@assets/images/inline_demo.gif";

export default function Page() {
  return (
    <div className="flex flex-col">
      <UserPrefView intro={intro} />
      <div className="flex flex-col gap-4 py-4">
        <h2 className="font-bold text-medium text-zinc-400 mt-2 leading-none">
          How To
        </h2>
        <p className="font-light leading-snug">
          Receive AI-generated completions as you type. Press <Kbd>→</Kbd> to
          accept. Currently <Code>zsh</Code> only.
        </p>
      </div>
      <Terminal title={"Inline shell completions"} className="my-2">
        <Terminal.Tab>
          <img src={inlineDemo} alt="Inline shell completion demo" />
        </Terminal.Tab>
      </Terminal>
      <UserPrefView array={settings}></UserPrefView>
    </div>
  );
}
