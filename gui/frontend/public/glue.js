var fn_login = async function (name) {
    console.log("invoke login(${name}");
}

var fn_create_account = async function (name) {
    console.log("invoke create_account(${name}");
}

if (typeof(window.__TAURI__) !== 'undefined') {
    const invoke = window.__TAURI__.invoke

    fn_login = async function (name) {
        return await invoke("login", {name: name});
    }

    fn_create_account = async function (name) {
        return await invoke("create_account", {name: name});
    }
}


export async function invokeLogin(name) {
    return await fn_login(name);
}

export async function invokeCreateAccount(name) {
    return await fn_create_account(name);
}


