import { CHAT_WIKI_URL, CLI_BINARY_NAME } from "@/lib/constants";

const chatSettings: { title: string }[] = [];

export default chatSettings;

export const intro = {
  title: "Chat",
  description: `Generative AI for your command line. Just run \`${CLI_BINARY_NAME} chat\`.`,
  link: CHAT_WIKI_URL,
  enable: {
    flag: "chat.disable",
    inverted: true,
    default: false,
  },
  disabled: true,
};

