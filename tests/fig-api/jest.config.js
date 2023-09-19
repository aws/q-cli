/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
  clearMocks: true,
  preset: "ts-jest",
  testEnvironment: "jsdom",
  setupFiles: ["<rootDir>/src/setup.ts"],
};
