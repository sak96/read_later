import pluginVue from 'eslint-plugin-vue';
import vueParser from 'vue-eslint-parser';
import parserTs from '@typescript-eslint/parser';
import type { Linter } from 'eslint';

export default [
  {
    files: ['src/**/*.vue'],
    ignores: ['node_modules', 'dist', '.git'],
    languageOptions: {
      ecmaVersion: 'latest' as const,
      sourceType: 'module' as const,
      parser: vueParser,
      parserOptions: {
        parser: parserTs
      },
      globals: {
        console: 'readonly' as const,
        window: 'readonly' as const,
        document: 'readonly' as const,
        fetch: 'readonly' as const
      }
    },
    plugins: {
      vue: pluginVue,
    },
    rules: {
      ...pluginVue.configs.base.rules,
      ...pluginVue.configs['vue3-essential'].rules,
      'vue/comment-directive': 'off',
      'vue/multi-word-component-names': 'off',
      'vue/no-side-effects-in-computed-properties': 'off',
      'no-unused-vars': 'off',
      'no-undef': 'off',
      'indent': ["error", 2],
    }
  },
  {
    files: ['src/**/*.ts'],
    ignores: ['node_modules', 'dist', '.git'],
    languageOptions: {
      ecmaVersion: 'latest' as const,
      sourceType: 'module' as const,
      parser: parserTs,
      globals: {
        console: 'readonly' as const,
        window: 'readonly' as const,
        document: 'readonly' as const,
        fetch: 'readonly' as const,
        vi: 'readonly' as const,
        describe: 'readonly' as const,
        it: 'readonly' as const,
        beforeEach: 'readonly' as const,
        afterEach: 'readonly' as const,
        MockInstance: 'readonly' as const,
        Mock: 'readonly' as const
      }
    },
    rules: {
      'no-unused-vars': 'off',
      'no-undef': 'off'
    }
  }
] satisfies Linter.Config[];
