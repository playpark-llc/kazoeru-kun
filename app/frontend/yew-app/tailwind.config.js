module.exports = {
    purge: {
        mode: "all",
        content: [
            "./src/**/*.rs",
            "./index.html",
            "./src/**/*.html",
            "./src/**/*.css",
        ],
    },
    theme: {
        extend: {
            height: {
                'screen': [
                '100vh','100dvh'
                ]
            },
            minHeight: {
                'screen': [
                '100vh','100dvh'
                ]
            },
            maxHeight: {
                'screen': [
                '100vh','100dvh'
                ]
            }
        },
    },
    variants: {},
    plugins: [],
};
