const {
    defineConfig,
    globalIgnores,
} = require("eslint/config");

const globals = require("globals");

const {
    fixupConfigRules,
} = require("@eslint/compat");

const tsParser = require("@typescript-eslint/parser");
const reactRefresh = require("eslint-plugin-react-refresh");
const js = require("@eslint/js");

const {
    FlatCompat,
} = require("@eslint/eslintrc");

const compat = new FlatCompat({
    baseDirectory: __dirname,
    recommendedConfig: js.configs.recommended,
    allConfig: js.configs.all
});

// Filter out globals with leading/trailing whitespace
const cleanGlobals = Object.fromEntries(
    Object.entries(globals.browser).filter(([key]) => key.trim() === key)
);

module.exports = defineConfig([
    globalIgnores(["**/dist", "**/.eslintrc.cjs", "**/vite.config.ts", "**/eslint.config.cjs"]),
    {
        languageOptions: {
            globals: {
                ...cleanGlobals,
            },

            parser: tsParser,
            ecmaVersion: "latest",
            sourceType: "module",

            parserOptions: {
                project: ["./tsconfig.json"],
                tsconfigRootDir: __dirname,
            },
        },

        extends: fixupConfigRules(compat.extends(
            "eslint:recommended",
            "plugin:@typescript-eslint/recommended",
            "plugin:react-hooks/recommended",
        )),

        plugins: {
            "react-refresh": reactRefresh,
        },

        rules: {
            "react-refresh/only-export-components": ["warn", {
                allowConstantExport: true,
            }],

            "no-undef": "error",
            "no-use-before-define": "error",
            "@typescript-eslint/no-unused-vars": "error",
            "@typescript-eslint/no-explicit-any": "warn",
            "@typescript-eslint/prefer-nullish-coalescing": "warn",
            "@typescript-eslint/prefer-optional-chain": "warn",
            "@typescript-eslint/no-non-null-assertion": "error",
            "@typescript-eslint/no-unsafe-call": "error",
            "@typescript-eslint/no-unsafe-member-access": "warn",
        },
    }
]);
