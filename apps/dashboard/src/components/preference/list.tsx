import { alphaByTitle } from "@/lib/sort";
import { Action, Pref, PrefDefault } from "@/types/preferences";
import { Setting } from "./listItem";
import { useEffect, useState } from "react";
import { Settings } from "@withfig/api-bindings";
import { getIconFromName } from "@/lib/icons";
import { Button } from "../ui/button";
import ExternalLink from "../util/external-link";
import { interpolateSettingBoolean } from "@/lib/utils";
import { useSetting } from "@/hooks/store/useSetting";

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
  const [setting, setSetting] = useSetting(intro.enable.flag);
  const [inputValue, setInputValue] = useState<PrefDefault>(intro.enable.default);
  const localValue = interpolateSettingBoolean(inputValue as boolean, intro.enable.inverted)
  
  // see if this specific setting is set in config file, then synchronize the initial state
  useEffect(() => {
    if (setting !== undefined) setInputValue(setting);
  }, [setting]);

  function toggleSwitch() {
    setSetting(!inputValue)
  }

  return (
    <section className="flex flex-col p-6 gap-4 w-full gradient-cw-secondary-light rounded-lg items-start text-white">
        <div className="flex gap-4">
          <div className="flex flex-col">
          {getIconFromName(intro.title, 48)}
          <h1 className="font-bold text-2xl font-ember leading-none">{intro.title}</h1>
          <p className="text-base">
            <span>{intro.description}</span>
            <ExternalLink
              href={intro.link}
              className="pl-1 text-white font-medium underline underline-offset-4 "
            >
              Learn more
            </ExternalLink>
          </p>
          </div>
        </div>
        <div className="flex flex-col gap-2">
          <Button variant="glass" className={`${localValue && 'text-white/50 border-white/50 hover:text-white'}`} onClick={toggleSwitch}>{localValue ? 'Enabled' : 'Enable'}</Button>
          {/* <Switch onClick={toggleSwitch} checked={localValue as boolean} /> */}
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
  const [viewDisabled, setViewDisabled] = useState<string | undefined>();
  const localDisabled = intro?.enable.inverted ? !viewDisabled : viewDisabled

  useEffect(() => {
    if (!intro?.enable) return;

    Settings.get(intro.enable.flag)
      .then((r) => {
        if (!r || r.jsonBlob === undefined) return;

        setViewDisabled(JSON.parse(r.jsonBlob));
      })
      .catch(() => {
        // Errors are thrown every time a setting isn't yet configured
        // so we just swallow those since they'll be set to the default automatically
        return
      });
  }, [intro, intro?.enable.flag]);

  return (
    <>
      {intro && <FeatureIntro intro={intro} />}
      {children}
      {array.map((section, i) => (
        <UserPrefSection
          disabled={localDisabled === 'true'}
          data={section}
          index={i}
          key={i}
        />
      ))}
    </>
  );
}
