import React from "react";
import {
  Suggestion,
  SuggestionType,
} from "@aws/amazon-q-developer-cli-shared/internal";
import { IconName, ICONS } from "../fig/icons";
import { useClassName } from "../state/style";

type SuggestionIconProps = {
  suggestion: Suggestion;
  iconPath: string;
  style: React.CSSProperties;
  isWeb: boolean;
};

type Icon =
  | EmojiIcon
  | TextIcon
  | UrlIcon
  | PresetIcon
  | PathIcon
  | TemplateIcon
  | UnknownIcon;

type ImgIcon = UrlIcon | PresetIcon | PathIcon | TemplateIcon | UnknownIcon;

interface EmojiIcon {
  type: "emoji";
  text: string;
}

interface TextIcon {
  type: "text";
  text: string;
}

interface UrlIcon {
  type: "url";
  url: URL;
}

interface PresetIcon {
  type: "preset";
  icon: string;
  fileType?: "png" | "svg";
}

interface PathIcon {
  type: "path";
  figUrl?: URL;
  path: string;
  kind?: "file" | "folder";
}

interface TemplateIcon {
  type: "template";
  figUrl?: URL;
  color?: string;
  badge?: string;
  ty?: string;
}

interface UnknownIcon {
  type: "unknown";
  figUrl: URL;
}

function parseIcon(icon: string, iconPath: string): Icon {
  const emojiRegex = /^(\p{Emoji_Presentation}|\p{Extended_Pictographic})/gu;
  const uriRegex = /^[a-z]+:\/\//g;

  if (emojiRegex.test(icon)) {
    return {
      type: "emoji",
      text: icon,
    };
  } else if (uriRegex.test(icon)) {
    const url = new URL(icon);

    if (url.protocol !== "fig:") {
      return {
        type: "url",
        url: new URL(icon),
      };
    } else {
      if (url.host === "" || url.host === "path") {
        return {
          type: "path",
          figUrl: url,
          path: url.pathname,
        };
      } else if (url.host === "template") {
        const params = new URLSearchParams(url.search);
        const color = params.get("color") ?? undefined;
        const badge = params.get("badge") ?? undefined;
        const ty = params.get("type") ?? undefined;

        return {
          type: "template",
          figUrl: url,
          color,
          badge,
          ty,
        };
      } else if (url.host === "icon") {
        const params = new URLSearchParams(url.search);
        const icon = params.get("type") ?? "box";
        return {
          type: "preset",
          icon,
        };
      } else {
        return {
          type: "unknown",
          figUrl: url,
        };
      }
    }
  } else if (icon === "" && iconPath !== "") {
    return {
      type: "path",
      path: iconPath,
    };
  } else {
    return {
      type: "text",
      text: icon,
    };
  }
}

function cdnIcon(icon: IconName, fileType: "png" | "svg" = "png") {
  return new URL(
    `https://specs.q.us-east-1.amazonaws.com/icons/${icon}.${fileType}`,
  );
}

function transformIconUri(icon: ImgIcon, isWeb: boolean): URL | undefined {
  if (icon.type === "url") {
    return icon.url;
  }

  if (icon.type === "preset") {
    const type = icon.icon as IconName;
    if (type) {
      if (ICONS.includes(type)) {
        return cdnIcon(type, icon.fileType);
      }
    }
  }

  if (!isWeb && "figUrl" in icon && icon.figUrl) {
    const url = icon.figUrl;
    if (window?.fig?.constants?.os === "windows") {
      return new URL(
        `https://fig.${icon.type}${url.pathname}${url.search}${url.hash}`,
      );
    }

    return new URL(`fig://${icon.type}${url.pathname}${url.search}${url.hash}`);
  }

  if (icon.type === "path") {
    if (isWeb) {
      if (icon.kind === "folder") {
        return cdnIcon("folder", "svg");
      } else {
        return cdnIcon("file", "svg");
      }
    } else {
      return new URL(`fig://path/${icon.path}`);
    }
  }

  return cdnIcon("box");
}

function iconToString(icon: ImgIcon): string {
  if (icon.type === "path") {
    return `Icon for ${icon.path}`;
  } else if (icon.type === "template") {
    return `Template icon`;
  } else {
    return `Icon for ${icon.type}`;
  }
}

function IconImg({
  icon,
  height,
  isWeb,
}: {
  icon: ImgIcon;
  height: string | number;
  isWeb: boolean;
}) {
  const iconClassName = useClassName(
    "icon-img",
    "grid overflow-hidden bg-contain bg-no-repeat",
  );

  const isTemplate = icon.type === "template";

  // const color = isTemplate ? icon.color : undefined;
  const badge = isTemplate ? icon.badge : undefined;
  const url = transformIconUri(icon, isWeb);

  return (
    <div
      role="img"
      aria-label={iconToString(icon)}
      className={iconClassName}
      style={{
        height,
        width: height,
        minWidth: height,
        minHeight: height,
        fontSize: typeof height === "number" ? height * 0.6 : height,
        backgroundImage: isTemplate
          ? `url(${cdnIcon("template")})`
          : `url(${url})`,
      }}
    >
      {badge &&
        (isTemplate ? (
          <span
            className="place-self-center text-center text-white"
            style={{
              fontSize: typeof height === "number" ? height * 0.5 : height,
            }}
          >
            {badge}
          </span>
        ) : (
          <span
            className="flex h-2.5 w-2.5 place-content-center place-self-end bg-contain bg-no-repeat text-[80%] text-white"
            style={{
              backgroundImage: `url(${cdnIcon("template")})`,
            }}
          >
            {badge}
          </span>
        ))}
    </div>
  );
}

const SuggestionIcon = ({
  suggestion,
  iconPath,
  style,
  isWeb,
}: SuggestionIconProps) => {
  const { name, type } = suggestion;
  const icon = parseIcon(suggestion.icon ?? "", iconPath);
  let height = style.height;
  let img;

  // The icon is a Emoji or text if it is <4 length
  if (icon.type === "emoji") {
    if (typeof height === "number") {
      height *= 0.8;
    }
    img = (
      <span
        style={{
          fontSize: height,
        }}
        className="suggestion-icon relative right-[0.0625rem] pb-2.5"
      >
        {icon.text}
      </span>
    );
  } else if (icon.type === "text") {
    if (typeof height === "number") {
      height *= 0.8;
    }
    img = (
      <span
        style={{
          fontSize: height,
        }}
        className="relative right-[0.0625rem] pb-2.5"
      >
        {icon.text}
      </span>
    );
  } else if (icon.type === "url") {
    img = <IconImg icon={icon} isWeb={isWeb} height={height ?? 0} />;
  } else if (
    icon.type === "path" &&
    (suggestion.type === "file" || suggestion.type == "folder") &&
    isWeb
  ) {
    img = (
      <IconImg
        icon={{
          type: "preset",
          icon: suggestion.type,
          fileType: "svg",
        }}
        isWeb={isWeb}
        height={height ?? 0}
      />
    );
  } else {
    const srcMap: Partial<Record<SuggestionType, ImgIcon>> = {
      folder: { type: "path", path: `${iconPath}${name}`, kind: "folder" },
      file: { type: "path", path: `${iconPath}${name}`, kind: "file" },
      subcommand: { type: "preset", icon: "command" },
      option: { type: "preset", icon: "option" },
      shortcut: { type: "template", color: "3498db", badge: "💡" },
      "auto-execute": { type: "preset", icon: "carrot" },
      arg: { type: "preset", icon: "box" },
      mixin: { type: "template", color: "628dad", badge: "➡️" },
    };

    const src = (type && srcMap[type] ? srcMap[type] : undefined) ?? {
      type: "preset",
      icon: "box",
    };

    img = <IconImg icon={src} isWeb={isWeb} height={height ?? 0} />;
  }

  return (
    <div className="suggestion-icon-container" style={style}>
      {img}
    </div>
  );
};

export default SuggestionIcon;
