import esbuild from 'esbuild';

const watch = process.argv.includes('--watch');

const profile = process.argv.includes('--dev') ? 'dev' : 'release';

esbuild.build({
    entryPoints: ['./src/main.tsx'],
    outdir: 'www/build',
    format: 'esm',
    bundle: true,
    watch,
    logLevel: 'info',
    loader: {
        '.wasm': 'file',
    },
    sourcemap: 'linked',
    minify: profile === 'release',
});
