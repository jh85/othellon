let wasmModule = null;

export async function initWasm() {
    if (wasmModule) return wasmModule;
    const mod = await import('/pkg/othello_wasm.js');
    await mod.default();
    wasmModule = mod;
    return mod;
}

export function getWasm() {
    return wasmModule;
}
