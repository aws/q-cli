import { UserPrefView } from "@/components/preference/list";
import { Code } from "@/components/text/code";
import { Terminal } from "@/components/ui/terminal";
import settings, { intro } from "@/data/chat";
import chatWithContextDemo from "@assets/images/chat_with_context_demo.gif";

export default function Page() {
  return (
    <>
      <UserPrefView array={settings} intro={intro} />
      <section className="flex flex-col py-4">
        <h2
          id="subhead-chat-how-to"
          className="font-bold text-medium text-zinc-400 leading-none mt-2"
        >
          How To
        </h2>
        <div className="flex flex-col gap-6 mt-4">
          <p className="font-light leading-tight">
            Amazon Q is a generative AI-powered assistant tailored for your
            command line. Ask Amazon Q a question, and receive an in-depth
            answer.
          </p>
          <div className="flex flex-col">
            <p className="font-light leading-tight">
              To increase the fidelity of the response, you can pass context
              about your environment with:
            </p>
            <ul className="flex flex-col gap-0 list-disc ml-5">
              <li>
                <Code>@history</Code> to pass your shell history
              </li>
              <li>
                <Code>@git</Code> to pass information about your current git
                repository
              </li>
              <li>
                <Code>@env</Code> to pass your shell environment
              </li>
            </ul>
          </div>
          <Terminal title="Passing Context">
            <Terminal.Tab>
              <img src={chatWithContextDemo} alt="chat with context demo" />
            </Terminal.Tab>
          </Terminal>
        </div>
      </section>
    </>
  );
}
