import { alphaByTitle } from "@/lib/sort";
import { Action, Pref } from "@/types/preferences";
import { Setting } from "./listItem";
import { useEffect, useState } from "react";
import { State } from "@withfig/api-bindings";
import { Autocomplete } from "../svg/icons";
import { Switch } from "../ui/switch";

type PrefSection = {
  title: string;
  properties?: Pref[];
  actions?: Action[];
};

type Intro = {
  title: string;
  description: string;
  link: string;
  enable: {
    flag: string;
    inverted: boolean;
    default: boolean;
  };
};

function FeatureIntro({ intro }: { intro: Intro }) {
  const [inputValue, setInputValue] = useState<boolean | undefined>();
  const localValue = intro.enable.inverted ? !inputValue : inputValue;

  useEffect(() => {
    State.get(intro.enable.flag)
      .then((r) => {
        if (!r) return;

        setInputValue(r);
      })
      .catch((e) => console.error({ getPref: e }));
  }, [intro.enable.flag]);

  function toggleSwitch() {
    setInputValue(!inputValue);
    State.set(intro.enable.flag, localValue).catch((e) =>
      console.error({ stateSetError: e })
    );
  }

  return (
    <section className="flex flex-col p-8 gap-4 w-full gradient-cw-secondary-light rounded-lg items-start text-white">
        <div className="flex flex-col">
          <Autocomplete size={48} />
          <h1 className="font-bold text-2xl font-ember">{intro.title}</h1>
          <p className="text-base">
            <span>{intro.description}</span>
            <a
              href={intro.link}
              className="pl-1 text-white font-medium underline underline-offset-4 "
            >
              Learn more
            </a>
          </p>
        </div>
        <div className="flex flex-col gap-2">
          <Switch onClick={toggleSwitch} checked={localValue as boolean} />
        </div>
    </section>
  );
}

export function SectionHeading({
  children,
  index,
}: {
  children: React.ReactNode;
  index: number;
}) {
  return (
    <h2
      id={`subhead-${index}`}
      className="font-bold text-medium text-zinc-400 leading-none mt-2"
    >
      {children}
    </h2>
  );
}

export function UserPrefSection({
  data,
  index,
  disabled,
}: {
  data: PrefSection;
  index: number;
  disabled?: boolean;
}) {
  const list = data.properties ?? data.actions;

  return (
    <section
      className={`flex flex-col py-4 ${disabled && "opacity-30 select-none"}`}
    >
      <SectionHeading index={index}>{data.title}</SectionHeading>

      {list?.sort(alphaByTitle).map((p, i) => {
        if (p.popular) return;

        return <Setting data={p} key={i} disabled={disabled} />;
      })}
    </section>
  );
}

export function UserPrefView({
  array,
  children,
  intro,
}: {
  array: PrefSection[];
  children?: React.ReactNode;
  intro?: Intro;
}) {
  const [viewDisabled, setViewDisabled] = useState<boolean | undefined>();

  useEffect(() => {
    if (!intro?.enable) return;

    State.get(intro.enable.flag)
      .then((r) => {
        if (!r) return;

        setViewDisabled(r);
      })
      .catch((e) => console.error({ getPref: e }));
  }, [intro, intro?.enable.flag]);

  return (
    <>
      {intro && <FeatureIntro intro={intro} />}
      {children}
      {array.map((section, i) => (
        <UserPrefSection
          disabled={viewDisabled}
          data={section}
          index={i}
          key={i}
        />
      ))}
    </>
  );
}
