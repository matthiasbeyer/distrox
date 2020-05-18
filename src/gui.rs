use std::fmt::Debug;

use anyhow::Error;
use anyhow::Result;
use web_view::WVResult;
use web_view::WebView;

use crate::app::App;
use crate::cli::*;
use crate::configuration::Configuration;
use crate::types::util::*;

pub fn run_gui(config: Configuration, adr: String) -> Result<()> {
    // GUI
    let app = {
        let device_name = config.get_device_name();
        let device_key  = config.get_device_key();

        if let (Some(name), Some(key)) = (device_name, device_key) {
            let name        = IPNSHash::from(name.clone());
            let key         = key.clone();
            let api_url     = config.get_api_url().clone();
            let api_port    = config.get_api_port().clone();

            App::load(name, key, &api_url, api_port)
        } else {
            // ask user for name(s)
            // boot repository
            // load App object
            unimplemented!()
        }
    };

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

