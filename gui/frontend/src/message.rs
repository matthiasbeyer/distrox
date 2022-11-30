#[derive(Debug)]
pub enum Message {
    CreateAccount,
    StartLoggingIn,
    CreateAccountFailed(String),
    LoginSuccess(String),
    LoginFailed(String),
}
