/** @type {import('tailwindcss').Config} */
module.exports = {
  purge: {
    mode: "jit",
    content: ["./src/**/*.rs", "./index.html", "./static/**/*.css"],
  },
  theme: {
    extend: {
      fontFamily: {
        DotGothic: ['"DotGothic16"', "sans-serif"],
        ZenKakuGothic: ['"Zen Kaku Gothic New"', "sans-serif"],
      },
      backgroundImage: {
        none: "none",
        "primary-check":
          ["repeating-linear-gradient(90deg,#fff,#fff 1px,transparent 1px,transparent 20px)", "repeating-linear-gradient(0deg,#fff,#fff 1px,#C0E2D1 1px,#C0E2D1 20px)"],
      },
      height: {
        auto: "auto",
        screen: ["100vh", "100dvh"],
      },
      minHeight: {
        auto: "auto",
        screen: ["100vh", "100dvh"],
      },
      maxHeight: {
        auto: "auto",
        screen: ["100vh", "100dvh"],
      },
      "bg-primary": {
        check: [],
      },
    },
  },
  variants: {},
  plugins: [],
};
