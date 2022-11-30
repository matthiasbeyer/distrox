use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

pub struct Login;

#[derive(Properties, PartialEq)]
pub struct LoginProbs {
    pub input_ref: NodeRef,
    pub onclick: Callback<()>,
}

impl Component for Login {
    type Message = ();
    type Properties = LoginProbs;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        log::debug!("Login::update()");
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cb = ctx.props().onclick.clone();
        let onclick = move |_| {
            cb.emit(());
        };

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
                                <input ref={ctx.props().input_ref.clone()} class="input" type="text" />
                              </p>
                            </div>
                          </div>
                        </div>

                        <div class="field is-horizontal">
                          <div class="field-label"> </div>
                          <div class="field-body">
                            <div class="field">
                              <div class="control">
                                <button class="button is-primary" onclick={onclick}>
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
}
