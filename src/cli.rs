use clap::crate_authors;
use clap::crate_version;
use clap::App;

pub fn app<'a>() -> App<'a> {
    App::new("distrox")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Distributed social network")
}
