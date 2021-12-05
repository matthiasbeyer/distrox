use clap::crate_authors;
use clap::crate_version;
use clap::App;
use clap::Arg;

pub fn app<'a>() -> App<'a> {
    App::new("distrox")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Distributed social network")


        .subcommand(App::new("profile")
            .author(crate_authors!())
            .version(crate_version!())
            .about("Profile actions")

            .subcommand(App::new("create")
                .author(crate_authors!())
                .version(crate_version!())
                .about("Create profile")

                .arg(Arg::with_name("name")
                    .long("name")
                    .required(true)
                    .takes_value(true)
                    .value_name("NAME")
                    .about("Name of the profile")
                )
            )

            .subcommand(App::new("serve")
                .author(crate_authors!())
                .version(crate_version!())
                .about("Just serve the profile")

                .arg(Arg::with_name("name")
                    .long("name")
                    .required(true)
                    .takes_value(true)
                    .value_name("NAME")
                    .about("Name of the profile")
                )
            )
        )
}
