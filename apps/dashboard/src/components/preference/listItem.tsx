import { useEffect, useState } from "react";
import { Switch } from "../ui/switch";
import { State } from "@withfig/api-bindings";
import { Pref, PrefDefault } from "@/types/preferences";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { Input } from "../ui/input";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import Keystroke from "../ui/keystrokeInput";

export function Setting({ data, disabled }: { data: Pref, disabled?: boolean }) {
  const [inputValue, setInputValue] = useState<PrefDefault>(data.default);
  const localValue = data.inverted ? !inputValue : inputValue;
  const multiSelectValue = inputValue as string[]
  const keystrokeValue = inputValue as string[]

  // see if this specific setting is set in config file, then synchronize the initial state
  useEffect(() => {
    State.get(data.id)
      .then((r) => {
        if (!r) return;

        setInputValue(r);
      })
      .catch((e) => console.error({ getPref: e }));
  }, [data.id]);

  function toggleSwitch() {
    setInputValue(!inputValue);
    State.set(data.id, localValue).catch((e) =>
      console.error({ stateSetError: e })
    );
  }

  function toggleMultiSelect(option: string) {
    // console.log(option, multiSelectValue)
    if (multiSelectValue.includes(option)) {
      const index = multiSelectValue.indexOf(option)
      multiSelectValue.splice(index, 1)
      const updatedArray = multiSelectValue
      // console.log('new array looks like:', updatedArray)
      State.set(data.id, updatedArray)
        .then(() => setInputValue(updatedArray) )
        .catch((e) =>
          console.error({ stateSetError: e })
        );
      return
    }

    // console.log('adding', option, 'to', multiSelectValue)
    const updatedArray = [...multiSelectValue, option]
    State.set(data.id, updatedArray)
    .then(() => 
      {
        setInputValue(updatedArray)
        // console.log('new array:', updatedArray)
      }
    ).catch((e) =>
      console.error({ stateSetError: e })
    );
  }

  return (
    <div className={`flex p-4 ${data.type === 'keystrokes' ? "pl-0" : "pl-2"} gap-4`}>
      {(data.type !== 'keystrokes') && <div className="flex-none w-12">
        {data.type === "boolean" && (
          <Switch onClick={toggleSwitch} checked={localValue as boolean} disabled={disabled} />
        )}
      </div>}
      <div className="flex flex-col gap-1">
        <h3 className="font-medium leading-none">{data.title}</h3>
        {data.description && (
          <p className="font-light leading-tight text-sm">{data.description}</p>
        )}
        {data.example && (
          <p className="font-light leading-tight text-sm">{data.example}</p>
        )}
        {data.type !== "boolean" && (
          <div className="pt-1">
            {/* single value <select> menu */}
            {data.type === "select" && (
              <Select disabled={disabled}>
                <SelectTrigger className="w-60">
                  <SelectValue placeholder={data.default} />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {data.options?.map((o, i) => (
                      <SelectItem value={o} key={i}>
                        {o}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            )}
            {/* multi-value <select> menu */}
            {data.type === "multiselect" && (
              <div className="relative">
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline">Select options</Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent className="w-60">
                  {data.options?.map((o, i) => {
                    const included = multiSelectValue.includes(o) as boolean
                    // console.log(o, included)
                    return (
                      <DropdownMenuCheckboxItem 
                      key={i}
                      checked={included}
                      onCheckedChange={() => toggleMultiSelect(o)}
                    >
                      {o}
                    </DropdownMenuCheckboxItem>
                    )
                  })}
                </DropdownMenuContent>
              </DropdownMenu>
              </div>
            )}
            {/* for number values, currently only used for ms, thus the 1000-unit step */}
            {data.type === "number" && (
              <Input
                disabled={disabled}
                type="number"
                step={1000}
                placeholder={
                  typeof data.default === "string"
                    ? data.default
                    : data.default?.toString()
                }
              />
            )}
            {/* generic text input */}
            {data.type === "text" && (
              <Input
                disabled={disabled}
                type="text"
                placeholder={
                  typeof data.default === "string"
                    ? data.default
                    : data.default?.toString()
                }
              />
            )}
            {/* multi-keystroke value input */}
            {data.type === "keystrokes" && <Keystroke values={keystrokeValue} setValues={setInputValue} />}
          </div>
        )}
      </div>
    </div>
  );
}
