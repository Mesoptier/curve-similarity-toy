import { instantiate } from './lib/rs_lib.generated.js';

instantiate({
    url: new URL('rs_lib_bg.wasm', document.location.href),
}).then(({ add }) => {
    console.log(add(1, 2));
});
