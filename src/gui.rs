use std::fmt::Debug;

use anyhow::Error;
use anyhow::Result;
use web_view::WVResult;
use web_view::WebView;

use crate::cli::*;
use crate::types::util::*;

pub fn run_gui(adr: String) -> Result<()> {
    let webview_content = web_view::Content::Url(format!("http://{}", adr));

    web_view::builder()
        .title("My Project")
        .content(webview_content)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(invoke_handler)
        .build()
        .map_err(Error::from)?
        .run()
        .map_err(Error::from)
}

fn invoke_handler<T: Debug>(webview: &mut WebView<T>, s: &str) -> WVResult {
    debug!("invoke-handler: {:?}, {:?}", webview, s);
    Ok(())
}

