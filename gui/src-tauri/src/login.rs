use crate::state::State;
use distrox_gui_types::LoginHandle;

#[tauri::command]
pub async fn login(_state: tauri::State<'_, State>, name: &str) -> Result<LoginHandle, String> {
    Ok(LoginHandle::new(name.to_string()))
}

#[tauri::command]
pub async fn create_account(
    _state: tauri::State<'_, State>,
    name: &str,
) -> Result<LoginHandle, String> {
    Ok(LoginHandle::new(name.to_string()))
}
