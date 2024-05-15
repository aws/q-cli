import { Intro } from "@/components/preference/list";
import { CHAT_WIKI_URL, CLI_BINARY_NAME } from "@/lib/constants";

const chatSettings: { title: string }[] = [];

export const intro: Intro = {
  title: "Chat",
  description: `Generative AI for your command line. Just run \`${CLI_BINARY_NAME} chat\`.`,
  link: CHAT_WIKI_URL,
};

export default chatSettings;
