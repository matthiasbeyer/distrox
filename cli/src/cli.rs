use clap::crate_authors;
use clap::crate_version;
use clap::App;
use clap::Arg;
use clap::ArgGroup;

pub fn app<'a>() -> App<'a> {
    App::new("distrox")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Distributed social network")
        .subcommand(
            App::new("profile")
                .author(crate_authors!())
                .version(crate_version!())
                .help("Profile actions")
                .subcommand(
                    App::new("create")
                        .author(crate_authors!())
                        .version(crate_version!())
                        .about("Create profile")
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .required(true)
                                .takes_value(true)
                                .value_name("NAME")
                                .help("Name of the profile"),
                        ),
                )
                .subcommand(
                    App::new("serve")
                        .author(crate_authors!())
                        .version(crate_version!())
                        .about("Just serve the profile")
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .required(true)
                                .takes_value(true)
                                .value_name("NAME")
                                .help("Name of the profile"),
                        )
                        .arg(
                            Arg::new("connect")
                                .long("connect")
                                .required(false)
                                .takes_value(true)
                                .multiple_occurrences(true)
                                .value_name("MULTIADDR")
                                .help("Connect to MULTIADDR as well"),
                        )
                        .arg(
                            Arg::new("listen")
                                .long("listen")
                                .required(false)
                                .takes_value(true)
                                .multiple_occurrences(true)
                                .value_name("MULTIADDR")
                                .help("Listen on MULTIADDR, e.g. '/ip4/127.0.0.1/tcp/10000'"),
                        ),
                )
                .subcommand(
                    App::new("cat")
                        .author(crate_authors!())
                        .version(crate_version!())
                        .about("Read complete timeline of profile")
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .required(true)
                                .takes_value(true)
                                .value_name("NAME")
                                .help("Name of the profile"),
                        ),
                )
                .subcommand(
                    App::new("post")
                        .author(crate_authors!())
                        .version(crate_version!())
                        .about("Just serve the profile")
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .required(true)
                                .takes_value(true)
                                .value_name("NAME")
                                .help("Name of the profile to post to"),
                        )
                        .arg(
                            Arg::new("editor")
                                .long("editor")
                                .short('e')
                                .required(false)
                                .takes_value(false)
                                .help("Launch the editor for the text to be posted")
                                .conflicts_with("text"),
                        )
                        .arg(
                            Arg::new("text")
                                .long("text")
                                .required(true)
                                .takes_value(true)
                                .value_name("TEXT")
                                .help("The text to be posted")
                                .conflicts_with("editor"),
                        )
                        .group(
                            ArgGroup::new("text-or-editor")
                                .args(&["text", "editor"])
                                .required(true), // one must be present
                        ),
                ),
        )
        .subcommand(
            App::new("watch")
                .author(crate_authors!())
                .version(crate_version!())
                .about("Watch events")
                .arg(
                    Arg::new("state_dir")
                        .index(1)
                        .required(true)
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .value_name("STATE_DIR")
                        .help("State dir for IPFS"),
                )
                .arg(
                    Arg::new("connect")
                        .long("connect")
                        .required(false)
                        .takes_value(true)
                        .multiple_occurrences(true)
                        .value_name("ADDR")
                        .help("Connect to this addr"),
                ),
        )
        .subcommand(
            App::new("gui")
                .author(crate_authors!())
                .version(crate_version!())
                .about("Start the distrox gui"),
        )
}
