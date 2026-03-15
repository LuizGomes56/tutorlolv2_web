const plugin = require("tailwindcss/plugin");

module.exports = {
    content: ["./index.html", "./src/**/*.rs"],
    plugins: [
        plugin(function ({ matchUtilities, theme }) {
            const zinc = theme("colors.zinc") || {};

            matchUtilities(
                {
                    "bg-std": (v) => ({ backgroundColor: v }),
                    "text-std": (v) => ({ color: v }),
                    "border-std": (v) => ({ borderColor: v }),
                },
                {
                    values: zinc,
                    type: "color"
                }
            );
        }),
    ],
};