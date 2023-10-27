import { cn } from "@/lib/utils";
import { useCallback, useState } from "react";
import { Link } from "react-router-dom";
import { Process } from "@withfig/api-bindings";
import autocompleteDemo from "@assets/images/autocomplete_demo.gif";
import aiDemo from "@assets/images/ai_demo.gif";

// const tasks = [
//   {
//     title: "Configure your system",
//     description:
//       "Set up accessibility, input methods, and log in with your Builder ID.",
//     check: () => true,
//   },
//   {
//     title: "Choose a theme",
//     description: "Set the CodeWhisperer theme to fit your terminal",
//     check: () => false,
//   },
//   {
//     title: "Customize tab behavior",
//     description: "",
//     check: () => true,
//   },
//   {
//     title: "Run `cw ai`",
//     description: "Convert natural language into Bash commands.",
//     check: () => false,
//   },
// ];

// type Task = {
//   title: string;
//   description?: string;
//   check?: () => boolean;
// };

// function Task({ task }: { task: Task; updateProgress: () => void }) {
//   // const taskComplete = task.check() === true

//   // useEffect(() => {
//   // updateProgress()
//   // }, [taskComplete, updateProgress])

// function parseToCode(string: string) {
//   const strArray = string.split('`')

//   if (string.charAt(0) === '`') {
//     strArray.map((s, i) => {
//       if (i % 2 !== 0) {
//         return <Code key={i}>{s}</Code>
//       } else {
//         return <span key={i}>{s}</span>
//       }
//     })
//   } else {
//     strArray.map((s, i) => {
//       if (i % 2 !== 0) {
//         return <span key={i}>{s}</span>
//       } else {
//         return <Code key={i}>{s}</Code>
//       }
//     })
//   }
// }

//   return (
//     <div className="flex p-4 pl-0">
//       {/* <div className="w-12 flex-none flex flex-col"></div> */}
//       <div className="flex flex-col">
//         <h2 className="font-bold">{task.title}</h2>
//         {task.description && <p>{task.description}</p>}
//       </div>
//     </div>
//   );
// }

export default function Page() {
  // const { setModal } = useContext(ModalContext);
  // const [tasksCompleted, setTasksCompleted] = useState(1);
  // const [completionPercentage, setCompletionPercentage] = useState(tasksCompleted / tasks.length * 100);

  // useEffect(() => {
  //   setCompletionPercentage(tasksCompleted / tasks.length * 100)
  // }, [tasksCompleted])

  const [activeTab, setActiveTab] = useState(0);

  const showTerminal = useCallback(() => {
    const script = `
    if [ -d "/Applications/iTerm.app" ]; then
        if ps aux | grep -i "[i]Term" > /dev/null; then
            osascript -e 'tell application "iTerm" to create window with default profile'
        else
            open -a "iTerm"
        fi
    elif [ -d "/Applications/Hyper.app" ]; then
        open -a "Hyper"
        sleep 1.5
        osascript -e 'tell application "System Events" to keystroke "n" using command down'
    else
        if ps aux | grep -i "Terminal.app" > /dev/null; then
            osascript -e 'tell application "Terminal" to do script ""'
        else
            open -a "Terminal"
        fi
    fi
    `;

    Process.run({
      executable: "bash",
      args: ["-c", script],
      environment: { PROCESS_LAUNCHED_BY_FIG: undefined },
    })
      .then(console.log)
      .catch(console.error);
  }, []);

  return (
    <div className="flex flex-col items-start gap-4">
      <div className="flex flex-col ">
        <h1 className="text-2xl font-bold select-none">Getting started</h1>
        {/* <p>Here is how to get the most out of CodeWhisperer.</p> */}
      </div>
      <div className="flex flex-col gap-2 w-full border rounded p-4 bg-neutral-50 mb-4">
        <h2 className="font-medium text-lg mb-2">
          Click{" "}
          <button className="font-bold underline" onClick={showTerminal}>
            here
          </button>{" "}
          to open a new terminal and autocomplete will just work!
        </h2>
        <ol className="flex flex-col">
          <li>
            {/* <div className="w-12 flex-none flex flex-col"></div> */}
            <h2 className="font-medium">
              {"• "}
              <span className="font-bold">Not working?</span>{" "}
              <Link
                to={"/help"}
                className="text-blue-500 underline decoration-1 underline-offset-1 hover:text-blue-800 hover:underline-offset-4 transition-all duration-100"
              >
                Help & support
              </Link>
            </h2>
          </li>
          <li>
            {/* <div className="w-12 flex-none flex flex-col"></div> */}
            <h2 className="font-medium">
              {"• "}
              <span className="font-bold">Customize settings?</span> Click the
              tabs on the left.
            </h2>
          </li>
        </ol>
      </div>
      <div className="place-self-center border rounded-lg border-neutral-800 w-full max-w-2xl">
        <div className="w-full h-auto rounded-[5px] flex flex-col bg-[#161A1D]">
          <div className="flex flex-row gap-1.5 p-2 bg-neutral-700 rounded-t">
            <div className="flex items-center justify-center w-3 h-3 rounded-full bg-red-500"></div>
            <div className="flex items-center justify-center w-3 h-3 rounded-full bg-yellow-500"></div>
            <div className="flex items-center justify-center w-3 h-3 rounded-full bg-green-500"></div>
          </div>
          <div className="grid grid-cols-2 border-b-neutral-950 border-b-2 gap-0.5">
            <div
              className={cn(
                "text-neutral-400 text-center p-1.5 hover:bg-neutral-800 hover:border-transparent transition-colors font-mono border-t border-neutral-950",
                activeTab == 0 &&
                  "bg-neutral-700 hover:bg-neutral-700 border-transparent text-neutral-100"
              )}
              onClick={() => setActiveTab(0)}
            >
              Autocomplete
            </div>
            <div
              className={cn(
                "text-neutral-400 text-center p-1.5 hover:bg-neutral-800 hover:border-transparent transition-colors font-mono border-t border-neutral-950",
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
