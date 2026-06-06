import js from "@eslint/js";
import globals from "globals";

export default [
  {
    ignores: ["**/third-party/**"]
  },
  js.configs.recommended,
  {
    languageOptions: {
      globals: globals.browser
    }
  }
];
