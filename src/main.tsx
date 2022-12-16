import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './components/App';

import init, { start } from '../rs_lib/pkg';
// @ts-ignore: esbuild is configured to export the filename of the .wasm file
import wasmFilePath from '../rs_lib/pkg/rs_lib_bg.wasm';

init(new URL(wasmFilePath, import.meta.url)).then(() => {
    const canvas = document.querySelector('canvas');
    const ctx = canvas.getContext('webgl2');

    start(ctx, 800, 800);
});

const container = document.getElementById('container');
const root = createRoot(container);

root.render(
    <StrictMode>
        <App />
    </StrictMode>,
);
