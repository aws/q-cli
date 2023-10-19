import { Check, Plus, X } from "lucide-react";
import {
  Dispatch,
  SetStateAction,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { PrefDefault } from "@/types/preferences";
import {
  VALID_CONTROL_KEYS, getKeyName, getKeySymbol
} from "@/lib/keybindings";
import ListenerContext from "@/context/input";

export default function Keystroke({
  id,
  values,
  setValues,
}: {
  id: string,
  values: string[],
  setValues: Dispatch<SetStateAction<PrefDefault>>;
}) {
  const { listening, setListening } = useContext(ListenerContext)
  const [inputValue, setInputValue] = useState<string[] | null>(null);
  const [isInvalid, setIsInvalid] = useState(false);
  const ref = useRef(null);

  const inputOpen = listening === id

  type keypressEvent = {
    key: string;
    keyCode: number;
    metaKey: boolean;
    ctrlKey: boolean;
    shiftKey: boolean;
    altKey: boolean;
    preventDefault: () => void;
    stopPropagation: () => void;
  };

  const handleKeyPress = useCallback((e: keypressEvent) => {
    console.log(e)
    const keys = new Set<string>();
    if (e.metaKey) keys.add("command");
    if (e.ctrlKey) keys.add("control");
    if (e.shiftKey) keys.add("shift");
    if (e.altKey) keys.add("option");
    const key = getKeyName(e.keyCode)

    const isInvalidCombination =
      keys.has("command") ||
      (keys.has("control") &&
        key !== "control" &&
        !VALID_CONTROL_KEYS.includes(key));
    setIsInvalid(isInvalidCombination);

    if (key) keys.add(key);
    setInputValue(Array.from(keys));
    e.preventDefault();
    e.stopPropagation();
  }, []);

  useEffect(() => {
    if (inputOpen) return

    setIsInvalid(false)
    setInputValue(null)
  }, [inputOpen])

  useEffect(() => {
    if (!inputOpen) return;
    if (!ref.current) return;
    // attach the event listener
    document.addEventListener("keydown", handleKeyPress);

    // remove the event listener
    return () => {
      document.removeEventListener("keydown", handleKeyPress);
    };
  }, [handleKeyPress, inputOpen]);

  function handleNewKeystroke() {
    if (!inputValue) {
      setListening(null);
      return;
    }

    if (isInvalid) return

    setValues([...values, inputValue.join("+")]);
    setListening(null);
    setInputValue(null);
  }

  function cancelKeystroke() {
    setInputValue(null);
    setListening(null);
  }

  function openInput() {
    setListening(id);
  }

  function removeKeybinding(index: number) {
    const workingArray = [...values]
    workingArray.splice(index, 1)

    setValues(workingArray)
  }

  return (
    <div className="flex flex-col gap-1">
    <div className="flex gap-2 flex-wrap">
      {values.map((k: string, i: number) => {
        return (
          <button
            onClick={() => removeKeybinding(i)}
            key={i}
            className="text-white/50 italic text-center text-xs flex justify-center gap-[2px] py-1 pl-[2px] group hover:bg-black hover:text-white rounded-md p-1 px-2 items-center pr-0 hover:pr-2 transition-all"
          >
            {k
              ? k.split("+").map((l, i) => (
                  <kbd
                    key={i}
                    className="p-1 py-[2px] not-italic text-black group-hover:text-white border border-black group-hover:border-white rounded-sm shadow-[0_4px_0_black] group-hover:shadow-[0_4px_0_white] relative -top-[2px]"
                  >
                    {getKeySymbol(l)}
                  </kbd>
                ))
              : "press keys"}
            <X className="h-3 group-hover:w-3 w-0 ml-1 opacity-0 group-hover:opacity-100 -translate-x-full group-hover:translate-x-0 hover:bg-black/5 transition-transform" />
          </button>
        );
      })}
      {inputOpen ? (
        <div className="flex gap-1">
          <button
            onClick={cancelKeystroke}
            className="p-1 px-2 text-black hover:text-white hover:bg-red-500 rounded-sm"
          >
            <X className="w-3 h-3" />
          </button>
          <div
            className={`flex items-stretch gap-1 p-[2px] py-1 ${(inputOpen && !inputValue) && "pl-3"} text-xs bg-black rounded-md`}
          >
            <div
              ref={ref}
              className="text-white/50 italic text-center flex justify-center items-center gap-[2px] rounded-sm"
            >
              {inputValue
                ? inputValue.map((k, i) => (
                    <kbd
                      key={i}
                      className="p-1 py-[2px] not-italic text-white border border-white rounded-sm shadow-[0_4px_0_white] relative -top-[2px]"
                    >
                      {k}
                    </kbd>
                  ))
                : "press keys"}
            </div>
            <button
              onClick={handleNewKeystroke}
              className="p-1 px-[6px] mx-[2px] text-white hover:bg-emerald-500 rounded-sm"
            >
              <Check className="w-3 h-3" />
            </button>
          </div>
        </div>
      ) : (
        <button
          onClick={openInput}
          className="p-1 px-[6px] hover:bg-black/5 rounded-lg"
        >
          <Plus className="h-3 w-3" />
        </button>
      )}
    </div>
    {isInvalid && <span className="text-xs font-medium text-red-500 pl-8">Sorry, that combination is invalid.</span>}
    </div>
  );
}
