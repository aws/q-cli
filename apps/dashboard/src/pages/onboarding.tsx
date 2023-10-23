import { useState } from "react";

const tasks = [
  {
    title: 'Open a new terminal emulator to get started'
  },
  {
    title: "If something's not working, check out Help & Support"
  },
  {
    title: "Customize your settings to make"
  }
  // {
  //   title: "Configure your system",
  //   description:
  //     "Set up accessibility, input methods, and log in with your Builder ID.",
  //   check: () => true,
  // },
  // {
  //   title: "Choose a theme",
  //   description: "Set the CodeWhisperer theme to fit your terminal",
  //   check: () => false,
  // },
  // {
  //   title: "Customize tab behavior",
  //   description: "",
  //   check: () => true,
  // },
  // {
  //   title: "Run `cw ai`",
  //   description: "Convert natural language into Bash commands.",
  //   check: () => false,
  // },
];

type Task = {
  title: string;
  description?: string;
  check?: () => boolean;
};

function Task({ task }: { task: Task, updateProgress: () => void}) {
  // const taskComplete = task.check() === true

  // useEffect(() => {
    // updateProgress()
  // }, [taskComplete, updateProgress])

  return (
    <div className="flex p-4">
      <div className="w-12 flex-none flex flex-col"></div>
      <div className="flex flex-col">
        <h2 className="font-bold">{task.title}</h2>
        {task.description && <p>{task.description}</p>}
      </div>
    </div>
  );
}

export default function Page() {
  // const { setModal } = useContext(ModalContext);
  const [tasksCompleted, setTasksCompleted] = useState(1)
  // const [completionPercentage, setCompletionPercentage] = useState(tasksCompleted / tasks.length * 100);

  // useEffect(() => {
  //   setCompletionPercentage(tasksCompleted / tasks.length * 100)
  // }, [tasksCompleted])

  // function startOnboarding() {
  //   setModal(<InstallModal />);
  // }

  return (
    <div className="flex flex-col items-start gap-4">
      <div className="flex flex-col ">
        <h1 className="text-2xl font-bold select-none">Getting started</h1>
        <p>
          Almost done. Just a few more tasks to help you get the most out of
          CodeWhisperer.
        </p>
      </div>
      <div className="flex flex-col w-full">
        <div className="py-4">
          {/* <span className="text-sm font-bold">{`${completionPercentage}% complete`}</span> */}
          {/* <Progress className="w-full" value={completionPercentage} /> */}
        </div>
        {tasks.map((t, i) => (
          <Task task={t} key={i} updateProgress={() => setTasksCompleted(tasksCompleted + 1)}/>
        ))}
      </div>
      {/* <Button
          onClick={startOnboarding}
          className="disabled:bg-zinc-400 h-auto py-2 px-6 mt-1"
        >
          Open onboarding
        </Button> */}
    </div>
  );
}
