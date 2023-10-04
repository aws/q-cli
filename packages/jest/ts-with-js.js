/** @type {import('ts-jest').JestConfigWithTsJest} */
const config = {
  testPathIgnorePatterns: ["/node_modules/", "/mocks/", "/fixtures/"],
  coveragePathIgnorePatterns: ["/mocks/", "/fixtures/"],
  preset: "ts-jest/presets/js-with-ts",
  noStackTrace: true,
  moduleNameMapper: {
    "^(\\.{1,2}/.*)\\.js$": "$1",
  },
};

export default config;
