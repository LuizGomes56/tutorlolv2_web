export default {
    content: [
        "./index.html",
        "./src/**/*.rs",
    ],
    plugins: [
        function ({ matchUtilities, theme }) {
            const zinc = theme('colors.zinc') || {};
            const values = Object.fromEntries(
                Object.entries(zinc).filter(([, v]) => typeof v === 'string')
            );
            matchUtilities(
                {
                    "bg-std": (v) => ({ backgroundColor: v }),
                    "text-std": (v) => ({ color: v }),
                    "border-std": (v) => ({ borderColor: v }),
                },
                { values }
            );
        },
    ],
}