const invoke = window.__TAURI_INVOKE__

export async function invokeLogin(name) {
    return await invoke("login", {name: name});
}

export async function invokeCreateAccount(name) {
    return await invoke("create_account", {name: name});
}


