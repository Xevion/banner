// For more info, see https://github.com/storybookjs/eslint-plugin-storybook#configuration-flat-config-format
import storybook from "eslint-plugin-storybook";

import path from "node:path";
import { includeIgnoreFile } from "@eslint/compat";
import js from "@eslint/js";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import tseslint from "typescript-eslint";
import svelteConfig from "./svelte.config.js";

const gitignorePath = path.resolve(import.meta.dirname, ".gitignore");

export default tseslint.config(
  includeIgnoreFile(gitignorePath),
  {
    ignores: ["dist/", ".svelte-kit/", "build/", "src/lib/bindings/", "scripts/"],
  }, // Base JS rules
  js.configs.recommended, // TypeScript: recommended type-checked + stylistic type-checked
  ...tseslint.configs.recommendedTypeChecked,
  ...tseslint.configs.stylisticTypeChecked, // Svelte recommended
  ...svelte.configs.recommended, // Global settings: environments + type-aware parser
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
        extraFileExtensions: [".svelte"],
      },
    },
    rules: {
      // typescript-eslint recommends disabling no-undef for TS projects
      // see: https://typescript-eslint.io/troubleshooting/faqs/eslint/#i-get-errors-from-the-no-undef-rule-about-global-variables-not-being-defined-even-though-there-are-no-typescript-errors
      "no-undef": "off",
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
    },
  }, // Svelte files: parser config + rule overrides
  {
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
        svelteConfig,
      },
    },
    rules: {
      "svelte/no-navigation-without-resolve": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-return": "off",
      "@typescript-eslint/no-unsafe-call": "off",
    },
  }, // Disable type-checked rules for plain JS config files
  {
    files: ["**/*.js"],
    ...tseslint.configs.disableTypeChecked,
  },
  storybook.configs["flat/recommended"]
);
