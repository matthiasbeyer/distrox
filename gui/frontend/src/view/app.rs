use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, NodeRef};

use crate::view::home::Home;
use crate::{message::Message, view::login::Login};

#[derive(Debug)]
pub enum App {
    Login { input_ref: NodeRef },
    CreateAccountFailed { err: String },
    LoggedIn { name: String },
    LoginFailed { err: String },
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self::Login {
            input_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("App::update(): {self:?}, msg => {msg:?}");

        match msg {
            Message::CreateAccount => match self {
                App::Login { input_ref } => {
                    log::debug!("Login view, logging in now...");
                    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                        let val = input.value();
                        log::info!("Creating new account: {}", val);

                        ctx.link().send_future(async move {
                            log::debug!("Calling create_account({val})");
                            match crate::tauri::call_create_account(val).await {
                                Ok(handle) => {
                                    log::error!("create_account() success: '{}'", handle.name());
                                    // When creating a new account, we immediately logging it in
                                    Message::LoginSuccess(handle.name().to_string())
                                }
                                Err(err) => {
                                    log::error!(
                                        "Error deserializing reply from create_account(): '{}'",
                                        err
                                    );
                                    Message::CreateAccountFailed(err.to_string())
                                }
                            }
                        });
                    } else {
                        log::error!("Cannot cast {input_ref:?} to HtmlInputElement");
                    }
                    true
                }
                _ => {
                    log::error!("Not in Login view");
                    true
                }
            },
            Message::StartLoggingIn => match self {
                App::Login { input_ref } => {
                    log::debug!("Login view, logging in now...");
                    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                        let val = input.value();
                        log::info!("Logging in: {}", val);

                        ctx.link().send_future(async move {
                            log::debug!("Calling login({val})");
                            match crate::tauri::call_login(val).await {
                                Ok(handle) => {
                                    log::error!("login() success: '{}'", handle.name());
                                    Message::LoginSuccess(handle.name().to_string())
                                }
                                Err(err) => {
                                    log::error!(
                                        "Error deserializing reply from login(): '{}'",
                                        err
                                    );
                                    Message::LoginFailed(err.to_string())
                                }
                            }
                        });
                    } else {
                        log::error!("Cannot cast {input_ref:?} to HtmlInputElement");
                    }
                    true
                }
                _ => {
                    log::error!("Not in Login view");
                    true
                }
            },

            Message::CreateAccountFailed(err) => {
                log::info!("CreateAccount: failed: {err}");
                *self = App::CreateAccountFailed { err };
                true
            }

            Message::LoginSuccess(name) => {
                log::info!("Login: Success!");
                *self = App::LoggedIn { name };
                true
            }

            Message::LoginFailed(err) => {
                log::info!("Login: failed: {err}");
                *self = App::LoginFailed { err };
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        log::info!("view()");

        match self {
            App::Login { input_ref } => {
                log::info!("view(): Login");

                let onclick_login = ctx.link().callback(move |_| {
                    web_sys::console::log_1(&"Login clicked".into());
                    Message::StartLoggingIn
                });

                let onclick_create = ctx.link().callback(move |_| {
                    web_sys::console::log_1(&"Create account clicked".into());
                    Message::CreateAccount
                });

                html! {
                    <Login {input_ref} {onclick_create} {onclick_login}/>
                }
            }

            App::CreateAccountFailed { err } => {
                html! {
                    <p> { "Creating account failed" } {err}</p>
                }
            }

            App::LoggedIn { .. } => {
                html! {
                    <Home />
                }
            }

            App::LoginFailed { err } => {
                html! {
                    <p> { "Login Failed " } {err} </p>
                }
            }
        }
    }
}
