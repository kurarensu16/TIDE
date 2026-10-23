/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        tide: {
          bg: 'var(--color-bg)',
          surface: 'var(--color-surface)',
          variant: 'var(--color-surface-variant)',
          border: 'var(--color-border-inactive)',
          'border-active': 'var(--color-border-active)',
          accent: 'var(--color-accent)',
          'accent-secondary': 'var(--color-accent-secondary)',
          success: 'var(--color-success)',
          warning: 'var(--color-warning)',
          danger: 'var(--color-danger)',
          text: 'var(--color-text-primary)',
          'text-secondary': 'var(--color-text-secondary)',
          muted: 'var(--color-text-muted)',
          'tab-active-bg': 'var(--color-tab-active-bg)',
          'tab-active-fg': 'var(--color-tab-active-fg)',
        },
      },
      fontFamily: {
        mono: [
          'JetBrains Mono',
          'Fira Code',
          'Cascadia Code',
          'Consolas',
          'ui-monospace',
          'monospace',
        ],
        sans: [
          'Geist',
          'Inter',
          '-apple-system',
          'BlinkMacSystemFont',
          'Segoe UI',
          'Roboto',
          'sans-serif',
        ],
      },
    },
  },
  plugins: [],
};
