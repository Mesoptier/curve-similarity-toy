import init, {add, get_enum, max, NumberEnum, sum} from '../rs_lib/pkg';
// @ts-ignore: esbuild is configured to export the filename of the .wasm file
import wasmFilePath from '../rs_lib/pkg/rs_lib_bg.wasm';

init(new URL(wasmFilePath, import.meta.url)).then(() => {
    console.log(add(1, 2));

    const items = new Int32Array([1, 2, 3, 4]);
    console.assert(sum(items) === 10);
    console.assert(max(items) === 4);
    console.assert(max(new Int32Array([])) === undefined);
    console.assert(get_enum('foo') === NumberEnum.Foo);
    console.assert(get_enum('bar') === NumberEnum.Bar);
    console.assert(get_enum('qux') === NumberEnum.Qux);
    console.assert(get_enum('asdf') === undefined);
});

console.log('Test');