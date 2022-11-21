export default {
  "contracts/**/*.rs": [
    "rustfmt"
  ],
  "tests/**/*.ts?(x)": [
    () => "tsc --project tsconfig.json --alwaysStrict",
  ],
  "tests/**/*.{js,jsx,ts,tsx}": [
    "prettier --write"
  ],
};
