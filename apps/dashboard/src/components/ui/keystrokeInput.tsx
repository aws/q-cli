import { Check, Plus, X } from "lucide-react"
import { Dispatch, SetStateAction, useCallback, useEffect, useRef, useState } from "react"
import { PrefDefault } from "@/types/preferences"
import { getKeyName, getKeySymbol } from "@/lib/keybindings"

export default function Keystroke ({values, setValues}: {values: string[], setValues: Dispatch<SetStateAction<PrefDefault>>}) {
  const [inputOpen, setInputOpen] = useState(false)
  const [inputValue, setInputvalue] = useState<string[] | null>(['ctrl', 'r'])
  const ref = useRef(null)

  type keypressEvent = {
    key: string,
    keyCode: number,
    preventDefault: () => void,
    stopPropagation: () => void
  }

  const handleKeyPress = useCallback((e: keypressEvent) => {
    const keystroke = new Set()
    
    console.log(`Key pressed: ${e.key}`);
    console.log({ keystroke })
    e.preventDefault()
    e.stopPropagation()

    keystroke.add(getKeySymbol(getKeyName(e.keyCode)))
  }, []);

  useEffect(() => {
    if (!inputOpen) return
    if (!ref.current) return
    // attach the event listener
    document.addEventListener('keydown', handleKeyPress);

    // remove the event listener
    return () => {
      document.removeEventListener('keydown', handleKeyPress);
    };
  }, [handleKeyPress, inputOpen]);

  function handleNewKeystroke() {
    if (!inputValue) {
      setInputOpen(false)
      return
    }

    setValues([...values, inputValue.join('+')])
    setInputOpen(false)
    setInputvalue(null)
  }

  function openInput() {
    setInputOpen(true)
  }

  return (
    <div className="flex gap-2 flex-wrap">
      {inputOpen
        ? <div className="flex gap-1">
          <button onClick={handleNewKeystroke} className="p-1 px-2 text-black hover:text-white hover:bg-red-500 rounded-sm"><X className="w-3 h-3"/></button>
          <div className={`flex items-stretch gap-1 p-[2px] py-1 text-xs bg-black rounded-md`}>
            <div ref={ref} className="text-white/50 italic text-center flex justify-center items-center gap-[2px] rounded-sm">
              {inputValue 
                ? inputValue.map((k, i) => <kbd key={i} className="p-1 py-[2px] not-italic text-white border border-white rounded-sm shadow-[0_4px_0_white] relative -top-[2px]">{k}</kbd>)
                : 'press keys'
              }
            </div>
            <button onClick={handleNewKeystroke} className="p-1 px-[6px] mx-[2px] text-white hover:bg-emerald-500 rounded-sm"><Check className="w-3 h-3"/></button>
          </div>
          </div>
        : <button onClick={openInput} className="p-1 px-[6px] hover:bg-black/5 rounded-lg"><Plus className="h-3 w-3" /></button>
      }
    {values.map((k: string, i: number) => {
      return(
        <button onClick={() => setValues(values.slice(i, 1))} key={i} className="text-white/50 italic text-center text-xs flex justify-center gap-[2px] py-1 pl-[2px] group hover:bg-black hover:text-white rounded-md p-1 px-2 items-center pr-0 hover:pr-2 transition-all">
              {k 
                ? k.split('+').map((l, i) => <kbd key={i} className="p-1 py-[2px] not-italic text-black group-hover:text-white border border-black group-hover:border-white rounded-sm shadow-[0_4px_0_black] group-hover:shadow-[0_4px_0_white] relative -top-[2px]">{l}</kbd>)
                : 'press keys'
              }
          <X className="h-3 group-hover:w-3 w-0 ml-1 opacity-0 group-hover:opacity-100 -translate-x-full group-hover:translate-x-0 hover:bg-black/5 transition-transform"/>
        </button>
      ) 
    })}
    </div>
  )
}