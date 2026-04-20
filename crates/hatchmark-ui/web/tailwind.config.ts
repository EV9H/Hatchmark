import type { Config } from 'tailwindcss';

export default {
  content: ['./src/**/*.{html,svelte,ts}'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter var', 'Inter', 'system-ui', 'sans-serif']
      },
      borderRadius: {
        xl: '1rem',
        '2xl': '1.25rem',
        '3xl': '1.75rem'
      },
      boxShadow: {
        glass: '0 8px 24px rgba(0,0,0,0.12)',
        glassHi: '0 16px 40px rgba(0,0,0,0.18)'
      },
      transitionTimingFunction: {
        spring: 'cubic-bezier(0.2, 0.8, 0.2, 1)'
      }
    }
  },
  plugins: []
} satisfies Config;
