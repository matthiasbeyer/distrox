extern crate ipfs_api;
extern crate chrono;
extern crate mime;
extern crate failure;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate uuid;
extern crate clap;
extern crate toml;
extern crate config;
extern crate hyper;
extern crate env_logger;

#[macro_use] extern crate is_match;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

mod cli_ui;
mod configuration;
mod repository;
mod state;
mod types;
mod version;

use std::collections::BTreeMap;
use std::str::FromStr;
use std::ops::Deref;
use std::sync::Arc;

use chrono::NaiveDateTime;
use futures::future::Future;
use serde_json::to_string_pretty as serde_json_to_string_pretty;
use serde_json::from_str as serde_json_from_str;

use configuration::Configuration;
use repository::Repository;
use types::block::Block;
use types::content::Content;
use types::content::Payload;
use types::util::IPFSHash;
use types::util::IPNSHash;
use types::util::MimeType;
use types::util::Timestamp;
use types::util::Version;

use std::process::exit;

fn main() {
    let _ = env_logger::init();
    debug!("Logger initialized");

    let boot = || -> (Configuration, Repository) {
        debug!("Loading configuration");
        let mut config = config::Config::default();
        config
            .merge(config::File::with_name("distrox"))
            .map_err(|e| {
                error!("Error loading config file: {:?}", e);
                exit(1);
            })
            .unwrap(); // safe!

        let config = config.try_into::<configuration::Configuration>()
            .map_err(|e| {
                error!("Error parsing config file: {:?}", e);
                exit(1);
            })
            .unwrap(); // safe!
        debug!("Config loaded");

        debug!("Loading repository...");
        let repo = Repository::new(config.api_url(), config.api_port())
            .map_err(|e| {
                error!("Error opening repository: {:?}", e);
                exit(1);
            })
            .unwrap(); // safe
        debug!("Repository loaded successfully!");

        (config, repo)
    };

    match cli_ui::build_ui()
        .get_matches()
        .subcommand()
    {

        ("gui", _mtch) => {
            error!("Not yet supported");
            exit(1)
        }

        ("is-block", Some(mtch)) => {
            debug!("Calling: is-block");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_block(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(|_| ()));
        }

        ("is-content", Some(mtch)) => {
            debug!("Calling: is-content");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(|_| ()));
        }

        ("is-post", Some(mtch)) => {
            debug!("Calling: is-post");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_post(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(|_| ()));
        }

        ("is-reply", Some(mtch)) => {
            debug!("Calling: is-reply");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_post(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(|content| {
                               let is_reply = match content.payload() {
                                   Payload::Post { reply_to, .. } => reply_to.is_some(),
                                   _ => false,
                               };

                               if !is_reply {
                                   exit(1)
                               }
                           }));
        }

        ("is-profile", Some(mtch)) => {
            debug!("Calling: is-profile");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_profile(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(|_| ()));
        }

        ("get-parent-blocks", Some(mtch)) => {
            debug!("Calling: get-parent-blocks");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_block(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(|block| block.parents().iter().for_each(|hash| {
                                   println!("{}", hash);
                           })));

        }

        ("get-devices", Some(mtch)) => {
            debug!("Calling: get-devices");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(|c| c.devices().iter().for_each(|d| println!("{}", d))));
        }

        ("get-payload-type", Some(mtch)) => {
            debug!("Calling: get-payload-type");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_post(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(|c| println!("{}", match c.payload() {
                               Payload::None                        => "None",
                               Payload::Post { .. }                 => "Post",
                               Payload::AttachedPostComments { .. } => "AttachedPostComments",
                               Payload::Profile { .. }              => "Profile",
                           })));
        }

        ("get-payload", Some(mtch)) => {
            debug!("Calling: get-payload");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_post(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .and_then(|c| serde_json_to_string_pretty(c.payload()))
                           .map_err(|e| {
                               error!("Error building JSON: {:?}", e);
                               exit(1)
                           })
                           .map(|j| println!("{}", j)));
        }

        ("get-post-content", Some(mtch)) => {
            debug!("Calling: get-post-content");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_post(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .and_then(move |content| {
                               match content.payload() {
                                   Payload::Post { content, content_format, .. } => {
                                       match (content_format.type_(), content_format.subtype()) {
                                           (mime::TEXT, _) => { // plain text will be printed
                                               repo.resolve_plain(&content)
                                                   .and_then(|blob| String::from_utf8(blob).map_err(Into::into))
                                                   .map_err(move |e| {
                                                       error!("Content is not UTF-8: {:?}", e);
                                                       exit(1)
                                                   })
                                                   .map(|blob| println!("{}", blob))
                                           },

                                           (_, _) => {
                                               error!("Cannot show mimetype of Post {hash} which lives at {content} ({mime})",
                                                      hash = hash,
                                                      content = content,
                                                      mime = content_format);
                                               exit(1)
                                           }
                                       }
                                   },
                                   _ => {
                                       error!("Not a Post");
                                       exit(1)
                                   }
                               }
                           }));
        }

        ("get-post-content-format", Some(mtch)) => {
            debug!("Calling: get-post-content-format");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_post(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(move |content| {
                               match content.payload() {
                                   Payload::Post { content_format, .. } => println!("{}", content_format),
                                   _ => {
                                       error!("Not a Post");
                                       exit(1)
                                   }
                               }
                           }));
        }

        ("get-post-reply-to", Some(mtch)) => {
            debug!("Calling: get-post-reply-to");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_post(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(move |content| {
                               match content.payload() {
                                   Payload::Post { reply_to, .. } => {
                                       reply_to.as_ref().map(|r| println!("{}", r));
                                   },
                                   _ => {
                                       error!("Not a Post");
                                       exit(1)
                                   }
                               }
                           }));
        }

        ("get-post-metadata", Some(mtch)) => {
            debug!("Calling: get-post-metadata");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_post(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(move |content| {
                               match content.payload() {
                                   Payload::Post {
                                       comments_will_be_propagated,
                                       comments_propagated_until,
                                       ..
                                   } => {
                                       comments_will_be_propagated.as_ref().map(|b| {
                                           println!("comments-will-be-propagated: {}", b);
                                       });
                                       comments_propagated_until.as_ref().map(|b| {
                                           println!("comments-will-be-propagated-until: {}", b);
                                       });
                                   },
                                   _ => {
                                       error!("Not a Post");
                                       exit(1)
                                   }
                               }
                           }));
        }

        ("get-profile-names", Some(mtch)) => {
            debug!("Calling: get-profile-names");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_profile(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(move |content| {
                               match content.payload() {
                                   Payload::Profile { names, .. } => {
                                       names.iter().for_each(|n| println!("{}", n));
                                   }
                                   _ => {
                                       error!("Not a Profile");
                                       exit(1)
                                   }
                               }
                           }));

        }

        ("get-profile-picture", Some(mtch)) => {
            debug!("Calling: get-profile-picture");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_profile(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(move |content| {
                               match content.payload() {
                                   Payload::Profile { picture, .. } => {
                                       picture.as_ref().map(|hash| {
                                           warn!("Showing picture on commandline not supported");
                                           warn!("Printing ipfs hash of picture");
                                           println!("{}", hash);
                                       });
                                   }
                                   _ => {
                                       error!("Not a Profile");
                                       exit(1)
                                   }
                               }
                           }));
        }

        ("get-profile-more", Some(mtch)) => {
            debug!("Calling: get-profile-more");
            let (_config, repo) = boot();
            let hash = IPFSHash::from(mtch.value_of("HASH").unwrap()); // safe by clap
            debug!("Working with hash: {}", hash);

            hyper::rt::run(repo
                           .resolve_content_profile(&hash)
                           .map_err(|e| {
                               let ignore_err = is_match!(e.downcast_ref(), Some(&ipfs_api::response::Error::Api(..)));

                               if !ignore_err {
                                   error!("Error running: {:?}", e);
                                   print_error_details(e);
                               }

                               exit(1)
                           })
                           .map(|content| {
                               match content.payload() {
                                   Payload::Profile { more, .. } => {
                                       match serde_json_to_string_pretty(more) {
                                           Err(e) => {
                                               error!("Error building JSON: {:?}", e);
                                               exit(1)
                                           },
                                           Ok(s) => {
                                               println!("{}", s);
                                           },
                                       }
                                   }
                                   _ => {
                                       error!("Not a Profile");
                                       exit(1)
                                   }
                               }
                           }));
        }

        ("create-post-blob", Some(mtch)) => {
            debug!("Calling: create-post-blob");
            let (_config, repo) = boot();

            let content = {
                let devices   = mtch.values_of("device").unwrap(); // safe by clap
                let timestamp = mtch.value_of("timestamp")
                    .map(|t| {
                        t.parse::<NaiveDateTime>()
                        .map(Timestamp::from)
                        .unwrap_or_else(|e| {
                            error!("Could not parse time: {:?}", e);
                            exit(1)
                        })
                    });

                let content_format = mtch.value_of("content-format").unwrap() // safe by clap
                        .parse::<mime::Mime>()
                        .map(MimeType::from)
                            .unwrap_or_else(|e| {
                            error!("Could not parse Mime: {:?}", e);
                            exit(1)
                        });

                let content_hash = mtch.value_of("content-hash").unwrap(); // safe by clap
                let replyto_hash = mtch.value_of("replyto-hash");

                let comments_will_be_propagated = mtch.value_of("comments-will-be-propagated")
                    .map(|b| b.parse::<bool>().unwrap_or_else(|e| {
                        error!("Could not parse bool: {:?}", e);
                        exit(1)
                    }));

                let comments_propagated_until   = mtch.value_of("comments-will-be-propagated-until")
                    .map(|b| b.parse::<NaiveDateTime>().map(Timestamp::from).unwrap_or_else(|e| {
                        error!("Could not parse time: {:?}", e);
                        exit(1)
                    }));

                let payload = Payload::Post {
                    content_format: content_format,
                    content: IPFSHash::from(String::from(content_hash)),
                    reply_to: replyto_hash.map(String::from).map(IPFSHash::from),
                    comments_will_be_propagated: comments_will_be_propagated,
                    comments_propagated_until: comments_propagated_until
                };

                let devices = devices.map(String::from).map(IPNSHash::from).collect();
                Content::new(devices, timestamp, payload)
            };

            hyper::rt::run({
                repo
                   .put_content(&content)
                   .map_err(|e| {
                       error!("Error running: {:?}", e);
                       print_error_details(e);
                       exit(1)
                   })
                   .map(|hash| println!("{}", hash))
            });
        }

        ("create-attached-post-comments-blob", Some(mtch)) => {
            debug!("Calling: create-attached-post-comments-blob");
            let (config, repo) = boot();

            let content = {
                let devices   = mtch.values_of("device").unwrap(); // safe by clap
                let timestamp = mtch.value_of("timestamp")
                    .map(|t| {
                        t.parse::<NaiveDateTime>()
                        .map(Timestamp::from)
                        .unwrap_or_else(|e| {
                            error!("Could not parse time: {:?}", e);
                            exit(1)
                        })
                    });

                let comments_for = mtch
                    .value_of("comments-for")
                    .map(String::from)
                    .map(IPFSHash::from)
                    .unwrap(); // safe by clap

                let refs = mtch.values_of("refs").unwrap() // safe by clap
                    .map(String::from)
                    .map(IPFSHash::from)
                    .collect();

                let payload = Payload::AttachedPostComments {
                    comments_for,
                    refs
                };

                let devices = devices.map(String::from).map(IPNSHash::from).collect();
                Content::new(devices, timestamp, payload)
            };

            hyper::rt::run({
                repo
                   .put_content(&content)
                   .map_err(|e| {
                       error!("Error running: {:?}", e);
                       print_error_details(e);
                       exit(1)
                   })
                   .map(|hash| println!("{}", hash))
            });
        }

        ("create-profile-blob", Some(mtch)) => {
            debug!("Calling: create-profile-blob");
            let (config, repo) = boot();

            let content = {
                let devices   = mtch.values_of("device").unwrap(); // safe by clap
                let timestamp = mtch.value_of("timestamp")
                    .map(|t| {
                        t.parse::<NaiveDateTime>()
                        .map(Timestamp::from)
                        .unwrap_or_else(|e| {
                            error!("Could not parse time: {:?}", e);
                            exit(1)
                        })
                    });

                let names = mtch.values_of("names").unwrap().map(String::from).collect(); // safe by clap
                let picture = if let Some(pic) = mtch.value_of("picture") {
                    Some(IPFSHash::from(String::from(pic)))
                } else {
                    None
                };
                let more = if let Some(more) = mtch.value_of("more") {
                    let json : BTreeMap<String, types::content::Userdata> = serde_json_from_str(more)
                        .unwrap_or_else(|e| {
                            error!("Error: {:?}", e);
                            exit(1)
                        });

                    Some(json)
                } else {
                    None
                };

                let payload = Payload::Profile {
                    names, picture, more: more.unwrap_or_else(|| BTreeMap::new())
                };

                let devices = devices.map(String::from).map(IPNSHash::from).collect();
                Content::new(devices, timestamp, payload)
            };

            hyper::rt::run({
                repo
                   .put_content(&content)
                   .map_err(|e| {
                       error!("Error running: {:?}", e);
                       print_error_details(e);
                       exit(1)
                   })
                   .map(|hash| println!("{}", hash))
            });
        }

        ("create-block-blob", Some(mtch)) => {
            debug!("Calling: create-block-blob");
            let (config, repo) = boot();

            let block = {
                let version = usize::from_str(mtch.value_of("version").unwrap()) // safe by clap
                    .map(Version::from)
                    .unwrap_or_else(|e|  {
                        error!("Could not parse version: {:?}", e);
                        exit(1)
                    });
                let parents = if let Some(parents) = mtch.values_of("parents") {
                    parents.map(String::from).map(IPFSHash::from).collect()
                } else {
                    vec![]
                };

                let content = mtch.value_of("content").map(String::from).map(IPFSHash::from).unwrap(); // safe by clap

                Block::new(version, parents, content)
            };

            hyper::rt::run({
                repo
                   .put_block(&block)
                   .map_err(|e| {
                       error!("Error running: {:?}", e);
                       print_error_details(e);
                       exit(1)
                   })
                   .map(|hash| println!("{}", hash))
            });
        }

        ("create-profile", Some(mtch)) => {
            debug!("Calling: create-profile");
            let (config, repo) = boot();

            let name      = mtch.value_of("name").map(String::from).unwrap(); // safe by clap
            let keyname   = format!("distrox-{}", name);
            let timestamp = Timestamp::from(::chrono::offset::Local::now().naive_local());
            let payload   = Payload::Profile {
                names:   vec![name],
                picture: None,
                more:    BTreeMap::new(),
            };
            let profile   = Content::new(vec![], Some(timestamp), payload);

            hyper::rt::run({
                // use ipfs defaults for lifetime and ttl
                repo.new_profile(keyname, profile, None, None)
                   .map_err(|e| {
                       error!("Error running: {:?}", e);
                       print_error_details(e);
                       exit(1)
                   })
                    .map(|(profile_name, profile_key)| {
                        println!("{}, {}", profile_name.deref(), profile_key.deref());
                    })
            });
        },

        ("post", Some(mtch)) => {
            use repository::ProfileName;

            debug!("Calling: post");
            let (config, repo) = boot();
            let repo = Arc::new(repo);
            let publish_key_name = mtch
                .value_of("profile_name")
                .map(String::from)
                .map(ProfileName::from)
                .unwrap(); // safe by clap
            let text = mtch
                .value_of("text")
                .map(String::from)
                .unwrap(); // safe by clap
            let time = ::chrono::offset::Local::now().naive_local();

            let repo2 = repo.clone();

            hyper::rt::run({
                repo.clone()
                    .get_key_id_from_key_name(publish_key_name.clone())
                    .and_then(move |key_id| {
                        let key_id = key_id.into();
                        repo2.deref_ipfs_hash(&key_id).map(|ipfs_hash| (key_id, ipfs_hash))
                    })
                    .and_then(move |(key_id, ipfs_hash)| {
                        repo.new_text_post(key_id, ipfs_hash, text, Some(time))
                    })
                    .map_err(|e| {
                        error!("Error running: {:?}", e);
                        print_error_details(e);
                        exit(1)
                    })
            });
        },

        ("publish", Some(mtch)) => {
            use repository::ProfileName;

            debug!("Calling: publish");
            let (config, repo) = boot();
            let repo = Arc::new(repo);
            let publish_key_name = mtch
                .value_of("profile_name")
                .map(String::from)
                .map(ProfileName::from)
                .unwrap(); // safe by clap
            let blockhash = mtch
                .value_of("blockhash")
                .map(String::from)
                .map(IPFSHash::from)
                .unwrap(); // safe by clap

            let repo2 = repo.clone();

            hyper::rt::run({
                repo.clone()
                    .get_key_id_from_key_name(publish_key_name)
                    .and_then(move |publish_key_id| {
                        repo.announce_block(publish_key_id, &blockhash, None, None)
                    })
                    .map_err(|e| {
                        error!("Error running: {:?}", e);
                        print_error_details(e);
                        exit(1)
                    })
            });
        },

        (other, _mtch) => {
            error!("Unknown command: {}", other);
            exit(1)
        }
    }
}

fn print_error_details(e: ::failure::Error) {
    error!(" backtrace: {}", e.backtrace());
    {
        let mut last_cause = Some(e.as_fail());
        while last_cause.is_some() {
            error!(" -> {}", last_cause.unwrap());
            last_cause = last_cause.unwrap().cause();
        }
    }
    match e.downcast().unwrap() {
        ipfs_api::response::Error::Client(e)                    => error!(" -> {}", e),
        ipfs_api::response::Error::Http(e)                      => error!(" -> {}", e),
        ipfs_api::response::Error::Parse(e)                     => error!(" -> {}", e),
        ipfs_api::response::Error::ParseUtf8(e)                 => error!(" -> {}", e),
        ipfs_api::response::Error::Url(e)                       => error!(" -> {}", e),
        ipfs_api::response::Error::Io(e)                        => error!(" -> {}", e),
        ipfs_api::response::Error::EncodeUrl(e)                 => error!(" -> {}", e),
        ipfs_api::response::Error::Api(e)                       => error!(" -> {}", e),
        ipfs_api::response::Error::StreamError(s)               => error!(" -> {}", s),
        ipfs_api::response::Error::UnrecognizedTrailerHeader(s) => error!(" -> {}", s),
        ipfs_api::response::Error::Uncategorized(s)             => error!(" -> {}", s),
    }
}

