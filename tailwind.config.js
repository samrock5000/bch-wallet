module.exports = {
  content: ["./src/**/*.{js,ts,jsx,tsx,mdx}"],
  daisyui: {
    styled: true,
    themes: true,
    base: true,
    utils: true,
    logs: true,
    rtl: false,
    prefix: "",
    darkTheme: "forest",
    theme: {
      container: {
        center: true,
           padding: '2rem',
      },
    },
  },

  plugins: [require("@tailwindcss/typography"), require("daisyui")],
};
