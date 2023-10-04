/** @type {import('eslint').Linter.Config} */
module.exports = {
  env: {
    es2021: true,
    node: true,
    jest: true,
  },
  extends: ["./index.js"],
  plugins: ["jest"],
};
