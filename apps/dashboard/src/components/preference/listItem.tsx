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

export function Setting({ data }: { data: Pref }) {
  const [inputValue, setInputValue] = useState<PrefDefault>(data.default);
  const localValue = data.inverted ? !inputValue : inputValue;

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

  return (
    <div className="flex p-4 pl-2 gap-4">
      <div className="flex-none w-12">
        {data.type === "boolean" && (
          <Switch onClick={toggleSwitch} checked={localValue as boolean} />
        )}
      </div>
      <div className="flex flex-col gap-1">
        <h2 className="font-medium text-base leading-none">{data.title}</h2>
        {data.description && (
          <p className="font-light leading-tight">{data.description}</p>
        )}
        {data.example && (
          <p className="font-light leading-tight">{data.example}</p>
        )}
        {data.type !== "boolean" && (
          <div className="pt-1">
            {data.type === "select" && (
              <Select>
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
            {data.type === "multiselect" && <div />}
            {data.type === "number" && <div />}
            {data.type === "text" && <div />}
            {data.type === "keystrokes" && <div />}
          </div>
        )}
      </div>
    </div>
  );
}
