import baseConfig from "./ts-with-js.js";

/** @type {import('ts-jest').JestConfigWithTsJest} */
const config = {
  ...baseConfig,
  testEnvironment: "jsdom",
  globals: {
    webkit: { messageHandlers: {} },
    fig: {},
  },
};

export default config;
