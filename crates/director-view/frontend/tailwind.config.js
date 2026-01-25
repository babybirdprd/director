/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            colors: {
                'director': {
                    'bg': '#0f0f0f',
                    'surface': '#1a1a1a',
                    'border': '#2a2a2a',
                    'accent': '#3b82f6',
                    'accent-hover': '#2563eb',
                    'text': '#e5e5e5',
                    'text-muted': '#737373',
                }
            },
            fontFamily: {
                'mono': ['JetBrains Mono', 'Fira Code', 'monospace'],
            }
        },
    },
    plugins: [],
}
