/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        primary: "#3a4851",
        "primary-focus": "#2c373f",
      },
    },
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: ["light", "dark"],
    themeRoot: ":root",
    prefix: "",
    logs: false,
    darkTheme: "dark",
    base: true,
    utils: true,
    rtl: false,
    styled: true,
  },
}