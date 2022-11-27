use yew::{html, Context, Html};

use crate::{app::App, message::Message};

pub fn login_screen(ctx: &Context<App>) -> Html {
    let login = ctx.link().callback(|_| Message::Login);

    html! {
        <div class="columns">
            <div class="column is-half is-offset-one-quarter">
                <div class="box">
                    <div class="field is-horizontal">
                      <div class="field-label is-normal">
                        <label class="label">{ "Username" }</label>
                      </div>
                      <div class="field-body">
                        <div class="field">
                          <p class="control">
                            <input class="input" type="text" />
                          </p>
                        </div>
                      </div>
                    </div>

                    <div class="field is-horizontal">
                      <div class="field-label"> </div>
                      <div class="field-body">
                        <div class="field">
                          <div class="control">
                            <button class="button is-primary" onclick={login}>
                              { "Login" }
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
