require("util").inspect.defaultOptions.depth = 5; // Increase AVA's printing depth

module.exports = {
  files: ["**/*.ava.ts"],
  extensions: ["ts"],
  require: ["ts-node/register"],
  failWithoutAssertions: false,
  timeout: "300000",
  failFast: true,
  verbose: true,
};
