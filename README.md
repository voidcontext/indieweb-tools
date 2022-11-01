# indieweb-tools

My collection of indieweb tools

## Components

- [app-auth](app-auth/): Oauth2 app authentication helper
- [janitor](janitor/): [TODO] clean up tool (removing test posts from social networks, etc.)
- [orion](orion/): Microblog syndication to Twitter and Mastodon
- [wormhole](wormhole/): [TODO] Url shortener

## Basic usage

1) Create a config file, i.e. `indieweb.toml`:

```toml
[rss]
urls = [ "http://example.com/rss.xml" ]

[db]
path = "indieweb.db"

[twitter]
# only the client id is required here, access and resfresh tokens should be stored in the db so that
# they can be updated
client_id = "your_client_id..."

[mastodon]
base_uri = "http://your-mastodon-instance.example.com"
access_token = "your_access_token..."

[wormhole]
protocol = "https"
domain = "short.domain"
```

2) Get Twitter auth tokens:

```bash
$ nix run .#app-auth -- --config indieweb.toml --db-path indieweb.db auth twitter
```

3) Syndicate posts to Twitter and Mastodon

```bash
$ nix run .#orion -- --config=indieweb.toml
```
