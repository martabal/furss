# furss

This tools aims to get full articles/texts from RSS feeds. Nowadays, most RSS feeds have only a small description and a link to the full article. This tool downloads the full article and put it back to the RSS feed.

## Run it

### Docker-cli ([click here for more info](https://docs.docker.com/engine/reference/commandline/cli/))

```sh
docker run --name=furss \
    -p 3000:3000 \
    ghcr.io/martabal/furss:latest
```

### Docker-compose

```yaml
services:
  immich:
    image: ghcr.io/martabal/furss:latest
    container_name: furss
    ports:
      - 3000:3000
    restart: unless-stopped
```

### Without docker

Make sure you have cargo and rust installed on you machine.

```sh
git clone https://github.com/martabal/furss.git
cargo build --release
./target/release/furss
```

## Roadmap

This tool only have basic functionalities, here are some features I want to implement:

- create a web interface to have a way to preview RSS feeds
- run the tool in CLI mode without a proxy
- more configuration (proxies, flaresolverr ...)

## Parameters

### Environment variables

|   Parameters   | Function                                            | Default Value |
| :------------: | --------------------------------------------------- | ------------- |
|   `-p 3000`    | Webservice port                                     |               |
| `-e APP_PORT`  | furss port (optional)                               | `3000`        |
| `-e LOG_LEVEL` | App log level (`DEBUG`, `INFO`, `WARN` and `ERROR`) | `INFO`        |
