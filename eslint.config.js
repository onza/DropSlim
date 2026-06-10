const js = require('@eslint/js')
const globals = require('globals')
const eslintConfigPrettier = require('eslint-config-prettier')

module.exports = [
  {
    ignores: [
      'dist/**',
      'dist-ui/**',
      'node_modules/**',
      'vendor/**',
      'src-tauri/target/**',
      'src-tauri/gen/**',
      'src-tauri/resources/**',
    ],
  },
  js.configs.recommended,
  {
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'commonjs',
      globals: {
        ...globals.node,
        ...globals.browser,
      },
    },
    rules: {
      'linebreak-style': ['error', 'unix'],
    },
  },
  {
    files: ['vite.config.js', '**/*.mjs'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
      globals: {
        ...globals.node,
      },
    },
  },
  {
    files: ['ui/**/*.js'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
      globals: {
        ...globals.browser,
      },
    },
  },
  eslintConfigPrettier,
]
