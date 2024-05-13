import { alphaByTitle } from "@/lib/sort";
import { Action, Pref, PrefDefault } from "@/types/preferences";
import { Setting } from "./listItem";
import { useEffect, useState } from "react";
import { Settings } from "@withfig/api-bindings";
import { getIconFromName } from "@/lib/icons";
import { cn, interpolateSettingBoolean } from "@/lib/utils";
import { useSetting } from "@/hooks/store/useSetting";
import { Switch } from "../ui/switch";
import { parseBackticksToCode } from "@/lib/strings";
import { Link } from "../ui/link";

export type PrefSection = {
  title: string;
  properties?: Pref[];
  actions?: Action[];
};

export type Intro = {
  title: string;
  description: string;
  link?: string;
  /**
   * Configuration settings for the feature flag. Only applies if `disabled`
   * is not true.
   */
  enable: {
    flag: string;
    inverted: boolean;
    default: boolean;
  };
  /**
   * Whether or not to allow the user to enable or disable the feature.
   */
  disabled?: boolean;
};

function FeatureIntro({ intro }: { intro: Intro }) {
  const [setting, setSetting] = useSetting(intro.enable.flag);
  const [inputValue, setInputValue] = useState<PrefDefault>(
    intro.enable.default,
  );
  const localValue = interpolateSettingBoolean(
    inputValue as boolean,
    intro.enable.inverted,
  );

  // see if this specific setting is set in config file, then synchronize the initial state
  useEffect(() => {
    if (setting !== undefined) setInputValue(setting);
  }, [setting]);

  function toggleSwitch() {
    setSetting(!inputValue);
  }

  return (
    <section className="flex flex-col p-6 gap-4 w-full gradient-q-secondary-light-alt rounded-lg items-start text-white">
      <div className="flex gap-4 justify-between w-full">
        <div className="flex gap-4">
          {getIconFromName(intro.title, 48)}
          <div className="flex flex-col">
            <h1 className="font-bold text-2xl font-ember leading-none">
              {intro.title}
            </h1>
            <p className="text-base">
              {parseBackticksToCode(
                intro.description,
                "!border-white !bg-white/20 !text-white py-[1px]",
              )}
              {intro.link && (
                <Link
                  href={intro.link}
                  className="pl-1 font-medium"
                  variant="primary"
                >
                  Learn more
                </Link>
              )}
            </p>
          </div>
        </div>
        {!intro.disabled && (
          <div className="flex items-center gap-2">
            <span className="font-bold">{localValue ? "On" : "Off"}</span>
            <Switch
              onClick={toggleSwitch}
              checked={localValue as boolean}
              variant={"inverted"}
            />
          </div>
        )}
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
      className={`flex flex-col gap-4 py-4 ${
        disabled && "opacity-30 select-none"
      }`}
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
  className,
}: {
  array: PrefSection[];
  children?: React.ReactNode;
  intro?: Intro;
  className?: string;
}) {
  const [viewDisabled, setViewDisabled] = useState<string | undefined>();
  const localDisabled = intro?.enable.inverted ? !viewDisabled : viewDisabled;

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
        return;
      });
  }, [intro, intro?.enable.flag]);

  return (
    <div className={cn("w-full flex flex-col", className)}>
      {intro && <FeatureIntro intro={intro} />}
      {children}
      {array.map((section, i) => (
        <UserPrefSection
          disabled={localDisabled === "true"}
          data={section}
          index={i}
          key={i}
        />
      ))}
    </div>
  );
}
