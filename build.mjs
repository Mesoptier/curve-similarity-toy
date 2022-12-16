import esbuild from 'esbuild';

const watch = process.argv.includes('--watch');

esbuild.build({
    entryPoints: ['./src/main.tsx'],
    outdir: 'www/build',
    format: "esm",
    bundle: true,
    watch,
    logLevel: 'info',
    loader: {
        '.wasm': 'file'
    },
    sourcemap: 'linked'
});