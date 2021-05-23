use clap::crate_authors;
use clap::crate_version;
use clap::App;
use clap::Arg;

pub fn app<'a>() -> App<'a> {
    App::new("distrox")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Distributed social network")

        .subcommand(App::new("create-profile")
            .author(crate_authors!())
            .version(crate_version!())
            .about("Create a new profile")

            .arg(Arg::with_name("content")
                .index(1)
                .multiple(false)
                .takes_value(true)
                .value_name("CONTENT")
                .help("The text posting as first profile content")
            )
        )

        .subcommand(App::new("post")
            .author(crate_authors!())
            .version(crate_version!())
            .about("Post to a profile")
            .arg(Arg::with_name("head")
                .index(1)
                .multiple(false)
                .takes_value(true)
                .value_name("HEAD")
                .help("Post with this HEAD as parent")
            )

            .arg(Arg::with_name("content")
                .index(2)
                .multiple(false)
                .takes_value(true)
                .value_name("TEXT")
                .help("Post this TEXT as text/text")
            )
        )

        .subcommand(App::new("get")
            .author(crate_authors!())
            .version(crate_version!())
            .about("Get block")
            .arg(Arg::with_name("head")
                .index(1)
                .multiple(false)
                .takes_value(true)
                .required(true)
                .value_name("HEAD")
                .help("Get HEAD block")
            )
        )
}
