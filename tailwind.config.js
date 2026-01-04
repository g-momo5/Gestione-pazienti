/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{svelte,js,ts}',
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#3B82F6',  // Blue 500 - più saturo
          light: '#60A5FA',    // Blue 400
          dark: '#2563EB',     // Blue 600
        },
        secondary: {
          DEFAULT: '#9CA3AF',  // Gray 400
          light: '#D1D5DB',
          dark: '#6B7280',
        },
        success: '#34D399',    // Green 400
        error: '#F87171',      // Red 400
        warning: '#FBBF24',    // Amber 400
        background: '#F3F4F6', // Gray 100 - leggermente più scuro
        surface: '#FFFFFF',    // White
        textPrimary: '#111827',   // Gray 900
        textSecondary: '#6B7280', // Gray 500
      },
      fontFamily: {
        sans: ['system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
      },
      fontSize: {
        'display-large': ['57px', { lineHeight: '64px', fontWeight: '400' }],
        'headline-large': ['32px', { lineHeight: '40px', fontWeight: '400' }],
        'title-large': ['22px', { lineHeight: '28px', fontWeight: '500' }],
        'body-large': ['16px', { lineHeight: '24px', fontWeight: '400' }],
        'label-large': ['14px', { lineHeight: '20px', fontWeight: '500' }],
      },
      borderRadius: {
        DEFAULT: '12px',
        'sm': '10px',
        'md': '14px',
        'lg': '18px',
        'xl': '24px',
        '2xl': '28px',
      },
      boxShadow: {
        'card': '0 2px 4px rgba(0, 0, 0, 0.1)',
        'card-hover': '0 4px 8px rgba(0, 0, 0, 0.15)',
        'modal': '0 8px 16px rgba(0, 0, 0, 0.2)',
      },
    },
  },
  plugins: [],
};
