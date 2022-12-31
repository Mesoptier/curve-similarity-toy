import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import init from '@rs_lib';
// @ts-ignore: esbuild is configured to export the filename of the .wasm file
import wasmFilePath from '@rs_lib/rs_lib_bg.wasm';

import { App } from './components/App';

const container = document.getElementById('container');
const root = createRoot(container);

init(new URL(wasmFilePath, import.meta.url)).then(() => {
    root.render(
        <StrictMode>
            <App />
        </StrictMode>,
    );
});
