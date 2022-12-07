import init from '../rs_lib/pkg';
// @ts-ignore: esbuild is configured to export the filename of the .wasm file
import wasmFilePath from '../rs_lib/pkg/rs_lib_bg.wasm';

init(new URL(wasmFilePath, import.meta.url)).then((module) => {
    console.log(module.add(1, 2));
});

console.log('Test');