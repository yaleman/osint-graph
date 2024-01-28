// loads the module which loads the WASM. It's loaders all the way down.
import init, { initSync } from '/static/osint_graph.js';
async function start() {
    await init('/static/osint_graph_bg.wasm');
    initSync();
}
start()
