var fn_login = async function (name) {
    console.log("invoke login(${name}");
}

if (typeof(window.__TAURI__) !== 'undefined') {
    const invoke = window.__TAURI__.invoke

    fn_login = async function (name) {
        return await invoke("login", {name: name});
    }
}


export async function invokeLogin(name) {
    return await fn_login(name);
}

