# Traefik ForwardAuth server for Cloudflare Teams/Access

![GitHub release (latest SemVer including pre-releases)](https://img.shields.io/github/v/release/Byte-Method/traefik-forwardauth-cloudflare-teams?include_prereleases&style=flat-square) ![GitHub](https://img.shields.io/github/license/Byte-Method/traefik-forwardauth-cloudflare-teams?style=flat-square) ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Byte-Method/traefik-forwardauth-cloudflare-teams/Docker?label=docker%20build&style=flat-square)

A really simple Docker container that runs a Flask server to validate the `Cf-Access-Jwt-Assertion` header from Cloudflare Access for authenticating requests using Traefik's `ForwardAuth` middleware.

## Getting started

You'll need two pieces of information:

* Your Cloudflare Teams domain. E.g.: `test.cloudflareaccess.com`
* Your Access application's audience tag (AUD).

Configure a ForwardAuth middleware address: `http://traefik-forwardauth-cloudflare-teams:8000/auth/{AUD}` (replace {AUD} with your audience tag).

The same container can be reused for multiple Access applications as long as they all use the same teams domain (since the audience tag is configured per-middleware).

## Build the image

```shell
docker build .
```

## docker-compose example

```yaml
services:
  traefik:
    image: "traefik:2"
    ports: ["443:443"]
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.api.rule=Host(`traefik.example.com`)"
      - "traefik.http.routers.api.service=api@internal"
      - "traefik.http.routers.api.entrypoints=https"
      - "traefik.http.routers.api.middlewares=cloudflare-access@docker"
      - "traefik.http.middlewares.cloudflare-access.forwardauth.address=http://traefik-forwardauth-cloudflare-teams:8000/auth/{AUD}"

  traefik-forwardauth-cloudflare-access:
    image: "ghcr.io/byte-method/traefik-forwardauth-cloudflare-teams:latest"
    environment:
      CF_TEAMS_DOMAIN: "test.cloudflareaccess.com"
```
