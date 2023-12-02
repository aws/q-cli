export type PrefDefault = unknown;

export type Pref = {
  id: string;
  title: string;
  description?: string;
  example?: string;
  type: string;
  inverted?: boolean;
  default: PrefDefault;
  popular?: boolean;
  options?: string[];
  icon?: string;
  background?: {
    color: string;
    image: string;
  };
};

export type Action = {
  id: string;
  title: string;
  description?: string;
  availability: string;
  type: string;
  default: string[];
  popular?: boolean;
};

export type RichText = {
  content: string;
  tag: string;
};

export type InstallCheck = {
  id: string;
  installKey?: "dotfiles" | "accessibility" | "inputMethod";
  title: string;
  description: string[];
  image?: string;
  action: string;
  explainer?: {
    title: string;
    steps: RichText[][];
  };
};

export type InstallCheckWithInstallKey = InstallCheck & {
  installKey: "dotfiles" | "accessibility" | "inputMethod";
};
