import esbuild from 'esbuild';
import { cleanPlugin } from 'esbuild-clean-plugin';
import { sassPlugin } from 'esbuild-sass-plugin';

const watch = process.argv.includes('--watch');

const profile = process.argv.includes('--dev') ? 'dev' : 'release';

esbuild.build({
    entryPoints: ['./src/main.tsx'],
    outdir: 'www/build',
    metafile: true,
    format: 'esm',
    bundle: true,
    watch,
    logLevel: 'info',
    loader: {
        '.wasm': 'file',
    },
    sourcemap: 'linked',
    minify: profile === 'release',
    plugins: [
        cleanPlugin(),
        sassPlugin(),
    ],
});
