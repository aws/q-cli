import { UserPrefView } from "@/components/preference/list";
import { Code } from "@/components/text/code";
import settings, { intro } from "@/data/chat";
import { cn } from "@/lib/utils";
import chatWithContextDemo from "@assets/images/chat_with_context_demo.gif";

export default function Page() {
  return (
    <>
      <UserPrefView array={settings} intro={intro} />
      <section className={`flex flex-col py-4`}>
        <h2
          id={`subhead-chat-how-to`}
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
          <div className="place-self-center border rounded-lg border-zinc-800 w-full max-w-2xl scale-75 relative -top-16 overflow-hidden">
            <div className="w-full h-auto rounded-lg flex flex-col bg-[#161A1D]">
              <div className="flex flex-row gap-1.5 p-2 bg-zinc-700 rounded-t">
                <div className="flex items-center justify-center w-3 h-3 rounded-full bg-red-500"></div>
                <div className="flex items-center justify-center w-3 h-3 rounded-full bg-yellow-500"></div>
                <div className="flex items-center justify-center w-3 h-3 rounded-full bg-green-500"></div>
              </div>
              <div className="grid grid-cols-1 border-b-zinc-900 border-b-2 gap-0.5">
                <div
                  className={cn(
                    "text-zinc-400 text-center p-1.5 hover:bg-zinc-800 hover:border-transparent transition-colors font-mono border-t border-zinc-950 select-none  cursor-pointer",
                    "bg-zinc-700 hover:bg-zinc-700 border-transparent text-zinc-100",
                  )}
                >
                  Passing Context
                </div>
              </div>
              <div className="p-2">
                <img src={chatWithContextDemo} alt="chat with context demo" />
              </div>
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
