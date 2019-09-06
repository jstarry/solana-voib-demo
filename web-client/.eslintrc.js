module.exports = {
  env: {
    browser: true,
    es6: true,
  },
  extends: [
    'eslint:recommended',
    'plugin:import/errors',
    'plugin:import/warnings',
    'plugin:import/react',
    'plugin:react/recommended',
    'plugin:prettier/recommended',
    'plugin:@typescript-eslint/eslint-recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:@typescript-eslint/recommended-requiring-type-checking',
  ],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    project: './tsconfig.json', // required for rules that need type information
    ecmaVersion: 2018,
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true
    }
  },
  plugins: [
    'react',
    '@typescript-eslint',
  ],
  overrides: [
    {
      'files': ['**/*.tsx'],
      'rules': {
        'react/prop-types': 'off'
      }
    }
  ],
  rules: {
    'no-trailing-spaces': ['error'],
    'import/first': ['error'],
    'import/no-commonjs': ['error'],
    'import/order': [
      'error',
      {
        groups: [
          ['internal', 'external', 'builtin'],
          ['index', 'sibling', 'parent'],
        ],
        'newlines-between': 'always',
      },
    ],
    indent: [
      'error',
      2,
      {
        MemberExpression: 1,
        SwitchCase: 1,
      },
    ],
    'linebreak-style': ['error', 'unix'],
    'no-console': [0],
    quotes: [
      'error',
      'single',
      {avoidEscape: true, allowTemplateLiterals: true},
    ],
    'prettier/prettier': 'error',
    'require-await': ['error'],
    semi: ['error', 'always'],
  },
  settings: {
    react: {
      version: 'detect',
    },
    'import/resolver': {
      node: {
        extensions: ['.ts','.tsx']
      }
    }
  },
};
