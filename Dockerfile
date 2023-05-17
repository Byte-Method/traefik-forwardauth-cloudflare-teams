#######################################
# Base image with defaults
#

FROM rust:slim AS builder

ARG APP_USER=app

WORKDIR /usr/src/traefik-forwardauth-cloudflare-teams
COPY . .

RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && apt-get autoremove -y && apt-get clean && rm -rf /var/lib/apt/lists/*

RUN cargo install --path .

#######################################
# Build image for runtime
#
FROM debian:bullseye-slim

ARG APP_USER=app

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && apt-get autoremove -y && apt-get clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/traefik-forwardauth-cloudflare-teams /usr/local/bin/traefik-forwardauth-cloudflare-teams

# copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

# create less privileged user
RUN useradd $APP_USER && \
	chmod +x /usr/local/bin/docker-entrypoint.sh

WORKDIR /usr/src/app

# don't run as root!
USER $APP_USER

# default port
EXPOSE 80

ENTRYPOINT [ "/usr/local/bin/docker-entrypoint.sh" ]

CMD ["traefik-forwardauth-cloudflare-teams", "--bind", "[::]:80"]
