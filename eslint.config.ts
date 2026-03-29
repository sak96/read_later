import { defineConfigWithVueTs, vueTsConfigs } from "@vue/eslint-config-typescript";
import vuePlugin from "eslint-plugin-vue";
import tseslint from "typescript-eslint";
import stylistic from "@stylistic/eslint-plugin";

export default defineConfigWithVueTs(
        // TypeScript base
        tseslint.configs.recommended,
        // tseslint.configs.recommendedTypeChecked,

        // Vue plugin flat config
        vuePlugin.configs["flat/recommended"],

        // Vue + TS integration
        vueTsConfigs.recommended,
        // vueTsConfigs.recommendedTypeChecked,

        // Stylistic rules (should come last)
        stylistic.configs.recommended,

        // Custom rule overrides
        {
                rules: {
                        'vue/multi-word-component-names': 'off', // disable multi-word component names globally
                },
        }
);
