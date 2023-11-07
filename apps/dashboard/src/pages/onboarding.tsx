import { cn } from "@/lib/utils";
import { useState } from "react";
import { Link } from "react-router-dom";
import autocompleteDemo from "@assets/images/autocomplete_demo.gif";
import aiDemo from "@assets/images/ai_demo.gif";

export default function Page() {
  const [activeTab, setActiveTab] = useState(0);

  return (
    <div className="flex flex-col items-start gap-4">
      <div className="flex flex-col ">
        <h1 className="text-2xl font-bold select-none">Getting started</h1>
      </div>
      <div className="flex flex-col gap-2 w-full border bg-slate-50 border-slate-200 rounded-lg p-4 mb-4">
      <h2 className="font-bold font-ember tracking-tight text-lg">Launch your terminal to start using autocomplete!</h2>

        <ol className="flex flex-col text-zinc-600">
          <li>
            <span className="text-sm flex items-baseline gap-1">
              <span className="font-semibold">Not working?</span>
              <span>Check out</span>
              <Link
                to={"/help"}
                className="text-blue-500 underline decoration-1 underline-offset-1 hover:text-blue-800 hover:underline-offset-4 transition-all duration-100"
              >
                Help & support
              </Link>
            </span>
          </li>
          <li>
            <span className="text-sm">
              <span className="font-semibold">Want to customize settings?</span> Use the tabs in the sidebar.
            </span>
          </li>
        </ol>
      </div>
      <div className="place-self-center border rounded-lg border-neutral-800 w-full max-w-2xl scale-75 relative -top-16">
        <div className="w-full h-auto rounded-[5px] flex flex-col bg-[#161A1D]">
          <div className="flex flex-row gap-1.5 p-2 bg-neutral-700 rounded-t">
            <div className="flex items-center justify-center w-3 h-3 rounded-full bg-red-500"></div>
            <div className="flex items-center justify-center w-3 h-3 rounded-full bg-yellow-500"></div>
            <div className="flex items-center justify-center w-3 h-3 rounded-full bg-green-500"></div>
          </div>
          <div className="grid grid-cols-2 border-b-neutral-950 border-b-2 gap-0.5">
            <div
              className={cn(
                "text-neutral-400 text-center p-1.5 hover:bg-neutral-800 hover:border-transparent transition-colors font-mono border-t border-neutral-950 select-none cursor-pointer",
                activeTab == 0 &&
                  "bg-neutral-700 hover:bg-neutral-700 border-transparent text-neutral-100"
              )}
              onClick={() => setActiveTab(0)}
            >
              Autocomplete
            </div>
            <div
              className={cn(
                "text-neutral-400 text-center p-1.5 hover:bg-neutral-800 hover:border-transparent transition-colors font-mono border-t border-neutral-950 select-none  cursor-pointer",
                activeTab == 1 &&
                  "bg-neutral-700 hover:bg-neutral-700 border-transparent text-neutral-100"
              )}
              onClick={() => setActiveTab(1)}
            >
              AI Translation
            </div>
          </div>
          <div className="p-2">
            {activeTab == 0 && (
              <img src={autocompleteDemo} alt="autocomplete demo" />
            )}
            {activeTab == 1 && <img src={aiDemo} alt="ai demo" />}
          </div>
        </div>
      </div>
    </div>
  );
}
