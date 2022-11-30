use distrox_gui_types::LoginHandle;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = invokeLogin, catch)]
    pub async fn login(name: String) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = invokeCreateAccount, catch)]
    pub async fn create_account(name: String) -> Result<JsValue, JsValue>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Tauri error: {:?}", .0)]
    Tauri(String),

    #[error(transparent)]
    Deser(#[from] serde_wasm_bindgen::Error),
}

pub async fn call_login(name: String) -> Result<LoginHandle, Error> {
    login(name)
        .await
        .map_err(|jsval| Error::Tauri(format!("{:?}", jsval)))
        .and_then(|val| serde_wasm_bindgen::from_value(val).map_err(Error::from))
}

pub async fn call_create_account(name: String) -> Result<LoginHandle, Error> {
    create_account(name)
        .await
        .map_err(|jsval| Error::Tauri(format!("{:?}", jsval)))
        .and_then(|val| serde_wasm_bindgen::from_value(val).map_err(Error::from))
}
