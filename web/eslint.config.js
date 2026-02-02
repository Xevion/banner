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
		ignores: ["dist/", ".svelte-kit/", "build/", "src/lib/bindings/"],
	},

	// Base JS rules
	js.configs.recommended,

	// TypeScript: recommended type-checked + stylistic type-checked
	...tseslint.configs.recommendedTypeChecked,
	...tseslint.configs.stylisticTypeChecked,

	// Svelte recommended
	...svelte.configs.recommended,

	// Global settings: environments + type-aware parser
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
		},
	},

	// Svelte files: parser config + rule overrides
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
		},
	},

	// Disable type-checked rules for plain JS config files
	{
		files: ["**/*.js"],
		...tseslint.configs.disableTypeChecked,
	},
);
