import betterTailwindcss from "eslint-plugin-better-tailwindcss";
import stylistic from "@stylistic/eslint-plugin";
import tsParser from "@typescript-eslint/parser";

export default [
  {
    ignores: ["dist/**", "wasm/**"],

    plugins: {
      "@stylistic": stylistic,
      "better-tailwindcss": betterTailwindcss,
    },

    settings: {
      "better-tailwindcss": {
        entryPoint: "src/index.css",
      },
    },

    rules: {
      "@stylistic/indent": ["error", 4, {
        ignoredNodes: ["JSXElement", "JSXExpressionContainer"],
        SwitchCase: 1,
      }],
      "@stylistic/comma-dangle": ["error", "always-multiline"],
      "@stylistic/no-trailing-spaces": "error",
      "@stylistic/object-curly-spacing": ["error", "always"],
      "@stylistic/array-bracket-spacing": ["error", "never"],
      "@stylistic/eol-last": ["error", "always"],
      "@stylistic/no-multi-spaces": "error",

      "better-tailwindcss/enforce-consistent-class-order": "error",
      "better-tailwindcss/no-duplicate-classes": "error",
      "better-tailwindcss/no-conflicting-classes": "warn",
    },
  },
  {
    files: ["**/*.{js,jsx,ts,tsx}"],
    languageOptions: {
      parser: tsParser,
      ecmaVersion: "latest",
      sourceType: "module",
      parserOptions: {
        ecmaFeatures: { jsx: true },
        project: ["./tsconfig.json"],
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
];
