import stylistic from "@stylistic/eslint-plugin"
import tsParser from "@typescript-eslint/parser"
import betterTailwindcss from "eslint-plugin-better-tailwindcss"

export default [
    {
        ignores: [
            "**/node_modules/**",
            "app/dist/**",
            "rusty_renju_web/wasm/**",
        ],
    },
    {
        files: ["app/**/*.{js,jsx,ts,tsx}", "rusty_renju_web/**/*.{js,jsx,ts,tsx}"],

        plugins: {
            "@stylistic": stylistic,
            "better-tailwindcss": betterTailwindcss,
        },

        settings: {
            "better-tailwindcss": {
                entryPoint: "app/src/index.css",
            },
        },

        rules: {
            "@stylistic/semi": ["error", "never"],
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
        files: ["app/**/*.{ts,tsx}", "rusty_renju_web/**/*.ts"],
        languageOptions: {
            parser: tsParser,
            ecmaVersion: "latest",
            sourceType: "module",
            parserOptions: {
                ecmaFeatures: { jsx: true },
                project: ["./app/tsconfig.json", "./rusty_renju_web/tsconfig.json"],
                tsconfigRootDir: import.meta.dirname,
            },
        },
    },
]
