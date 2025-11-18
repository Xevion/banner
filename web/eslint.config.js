import js from "@eslint/js";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default tseslint.config(
  // Ignore generated files and build outputs
  {
    ignores: ["dist", "node_modules", "src/routeTree.gen.ts", "*.config.js"],
  },
  // Base configs
  js.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  // React plugin configuration
  {
    files: ["**/*.{ts,tsx}"],
    plugins: {
      react,
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    languageOptions: {
      parserOptions: {
        project: true,
        tsconfigRootDir: import.meta.dirname,
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
    settings: {
      react: {
        version: "19.0",
      },
    },
    rules: {
      // React rules
      ...react.configs.recommended.rules,
      ...react.configs["jsx-runtime"].rules,
      ...reactHooks.configs.recommended.rules,

      // React Refresh
      "react-refresh/only-export-components": ["warn", { allowConstantExport: true }],

      // TypeScript overrides
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/no-explicit-any": "warn",

      // Disable prop-types since we're using TypeScript
      "react/prop-types": "off",
    },
  }
);
