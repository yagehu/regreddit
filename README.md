# Regreddit

[![Build Status](https://travis-ci.org/yagehu/regreddit.svg?branch=master)](https://travis-ci.org/yagehu/regreddit)

Regreddit is a command line tool that can delete everything
(posts and comments for now) from your Reddit account.

## Install

```
$ cargo install regreddit
```

## Credentials

You need a credentials file `.regreddit.toml` in the current working directory
for the commands to work. This is an example .regreddit.toml file with fake
credentials:

```yaml
[credentials]
client_id = "clientidstring"
secret = "secretstring"
username = "trsutyhardware"
password = "myveryweakpassword"
```

To obtain the client ID and secret, follow the steps outlined in the official
Reddit
[documentation](https://github.com/reddit-archive/reddit/wiki/OAuth2).
Select the "script app" type.

## Usage

### Delete everything

To delete all your comments and posts:

```
$ regreddit --yes
```

You can view the logs by:

```
$ regreddit --yes -vvv
```

## Why

Since USCIS checks foreign nationals for crimethink by requiring everyone to
hand over social media accounts, it may be best to sanitize your social media
starting with Reddit.

