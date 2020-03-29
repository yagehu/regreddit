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
use log4rs::append::console::ConsoleAppender;

static NAME: &str = "regreddit";
static VERSION: &str = "v0.1.0";
static AUTHOR_REDDIT_USERNAME: &str = "trustyhardware";

fn main() {
    let matches = clap::App::new("regreddit")
        .version(VERSION)
        .about("Nuke your Reddit account.")
        .author("Yage Hu <yagehu@qq.com>")
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
        .subcommand(
            clap::SubCommand::with_name("submit")
                .about("Submit a post.")
                .arg(
                    clap::Arg::with_name("type")
                        .required(true)
                        .possible_values(&["link", "self-post"]),
                )
                .arg(clap::Arg::with_name("subreddit").required(true))
                .arg(clap::Arg::with_name("title").required(true))
                .arg(
                    clap::Arg::with_name("url")
                        .long("url")
                        .help("The URL to submit.")
                        .required_if("type", "link")
                        .takes_value(true),
                ),
        )
        .get_matches();

    config_logger();

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
        let post_type = matches.value_of("type").unwrap();

        if post_type == "link" {
            match app.submit(app::SubmitParams {
                credentials: &settings.credentials,
                post_type: matches.value_of("type").unwrap(),
                subreddit: matches.value_of("subreddit").unwrap(),
                title: matches.value_of("title").unwrap(),
                url: matches.value_of("url"),
            }) {
                Ok(_res) => process::exit(0),
                Err(err) => {
                    eprintln!("Could not post to Reddit.");
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

fn config_logger() {
    let stderr = log4rs::append::console::ConsoleAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {h({l:>5})} {m}\n",
        )))
        .build();
    let config = log4rs::config::Config::builder()
        .appender(
            log4rs::config::Appender::builder()
                .build("stderr", Box::new(stderr)),
        )
        .build(
            log4rs::config::Root::builder()
                .appender("stderr")
                .build(log::LevelFilter::Debug),
        )
        .unwrap();
    let _ = log4rs::init_config(config).unwrap();
}
