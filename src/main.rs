mod app;
mod client;
mod error;
mod reddit;
mod settings;

#[macro_use]
extern crate serde_derive;

use std::process;

use clap;

use crate::app::{App, AppImpl, Params, RegredditParams};
use crate::client::ClientImpl;
use crate::settings::Settings;

static NAME: &str = "regreddit";
static VERSION: &str = "v0.1.0";
static AUTHOR_REDDIT_USERNAME: &str = "trustyhardware";

fn main() {
    let matches = clap::App::new("regreddit")
        .version(VERSION)
        .about("Nuke your Reddit account.")
        .author("Yage Hu <yagehu@qq.com>")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .arg(
            clap::Arg::with_name("yes")
                .long("yes")
                .help("The command won't do anything without this flag."),
        )
        .arg(
            clap::Arg::with_name("username")
                .long("username")
                .help("The username of the Reddit account."),
        )
        .arg(
            clap::Arg::with_name("verbosity")
                .short("v")
                .help("The verbosity of logging. Can be repeated `-vvv`")
                .multiple(true)
        )
        .subcommand(
            clap::SubCommand::with_name("submit")
                .about("Submit to Reddit.")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    clap::SubCommand::with_name("link")
                        .about("Submit a link.")
                        .arg(clap::Arg::with_name("subreddit").required(true))
                        .arg(clap::Arg::with_name("title").required(true))
                        .arg(
                            clap::Arg::with_name("url")
                                .help("The URL to submit.")
                                .required(true),
                        )
                )
                .subcommand(
                    clap::SubCommand::with_name("self-post")
                        .about("Submit a self-post.")
                        .arg(clap::Arg::with_name("subreddit").required(true))
                        .arg(clap::Arg::with_name("title").required(true))
                        .group(
                            clap::ArgGroup::with_name("content")
                                .args(&["text", "text-file"])
                                .required(true),
                        )
                        .arg(
                            clap::Arg::with_name("text")
                                .long("text")
                                .help("The body text to submit.")
                                .takes_value(true),
                        )
                        .arg(
                            clap::Arg::with_name("text-file")
                                .long("text-file")
                                .help("A file containing the body text to submit.")
                                .takes_value(true),
                        )
                        .arg(
                            clap::Arg::with_name("richtext_json")
                                .long("richtext-json")
                                .help("The body richtext JSON data to submit.")
                                .takes_value(true),
                        )
                        .arg(
                            clap::Arg::with_name("richtext_json_file")
                                .long("richtext-json-file")
                                .help("A file containing richtext JSON data to submit.")
                                .takes_value(true),
                        ),
        )
    )
    .get_matches();

    config_logger(matches.occurrences_of("verbosity"));

    let mut client = ClientImpl::new(client::Params {
        user_agent: format!(
            "{}/{} by /u/{}",
            NAME, VERSION, AUTHOR_REDDIT_USERNAME
        ),
    });
    let mut app = AppImpl::new(Params {
        client: &mut client,
    });
    let settings = Settings::new().unwrap();

    if let Some(matches) = matches.subcommand_matches("submit") {
        if let Some(matches) = matches.subcommand_matches("link") {
            match app.submit_link(app::SubmitLinkParams {
                credentials: &settings.credentials,
                subreddit: matches.value_of("subreddit").unwrap(),
                title: matches.value_of("title").unwrap(),
                url: matches.value_of("url").unwrap(),
            }) {
                Ok(_res) => process::exit(0),
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1)
                }
            }
        }

        if let Some(matches) = matches.subcommand_matches("self-post") {
            match app.submit_self_post(app::SubmitSelfPostParams {
                credentials: &settings.credentials,
                subreddit: matches.value_of("subreddit").unwrap(),
                title: matches.value_of("title").unwrap(),
                text: matches.value_of("text"),
                text_file: matches.value_of("text-file"),
                richtext_json: matches.value_of("richtext-json"),
                richtext_json_file: matches.value_of("richtext-json-file"),
            }) {
                Ok(_res) => process::exit(0),
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1)
                }
            }
        }
    }

    if !matches.is_present("yes") {
        eprintln!("You did not specify the `--yes` flag. Exiting...");
        process::exit(1);
    }

    match app.regreddit(RegredditParams {
        credentials: &settings.credentials,
    }) {
        Ok(_) => eprintln!("Successfully nuked your Reddit account."),
        Err(err) => {
            eprintln!("Error {:?}", err);
        }
    }
}

fn config_logger(verbosity: u64) {
    let stderr = log4rs::append::console::ConsoleAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {h({l:>5})} {m}\n",
        )))
        .build();

    let level_filter;

    match verbosity {
        0 => level_filter = log::LevelFilter::Off,
        1 => level_filter = log::LevelFilter::Error,
        2 => level_filter = log::LevelFilter::Warn,
        3 => level_filter = log::LevelFilter::Info,
        4 => level_filter = log::LevelFilter::Debug,
        _ => level_filter = log::LevelFilter::Trace,
    }

    let config = log4rs::config::Config::builder()
        .appender(
            log4rs::config::Appender::builder()
                .build("stderr", Box::new(stderr)),
        )
        .build(
            log4rs::config::Root::builder()
                .appender("stderr")
                .build(level_filter),
        )
        .unwrap();
    let _ = log4rs::init_config(config).unwrap();
}
