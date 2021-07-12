#!/usr/bin/env sh

set -e

if [ -z "$CF_TEAMS_DOMAIN" ]; then
	echo "CF_TEAMS_DOMAIN is not set, exiting..."
	exit 1
fi

exec "$@"
