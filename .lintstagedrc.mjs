export default {
  "contracts/**/*.rs": [
    "rustfmt"
  ],
  "**": [
    "npx cspell --words-only --unique"
  ]
};
