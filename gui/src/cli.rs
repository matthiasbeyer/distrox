use clap::crate_authors;
use clap::crate_version;
use clap::App;
use clap::Arg;

pub fn app<'a>() -> App<'a> {
    App::new("distrox-gui")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Distributed social network, GUI frontend")

        .arg(Arg::new("name")
            .index(1)
            .takes_value(true)
            .value_name("NAME")
            .about("Profile to load the GUI for")
        )
}

