from os import getenv
from jwt import decode
from jwt import PyJWKClient

CF_TEAMS_DOMAIN = getenv('CF_TEAMS_DOMAIN')
CF_ACCESS_CERTS_URL = f'https://{CF_TEAMS_DOMAIN}/cdn-cgi/access/certs'

JWT_DECODE_OPTIONS = {
    # Require expiry, issued-at, and email
    "require": ["exp", "iat", "email"],
}

def decode_token(jwt, audience):
    jwks_client = PyJWKClient(CF_ACCESS_CERTS_URL)
    signing_key = jwks_client.get_signing_key_from_jwt(jwt)

    return decode(
        jwt,
        key=signing_key.key,
        algorithms=["RS256"],
        options=JWT_DECODE_OPTIONS,
        audience=audience,
        issuer=f'https://{CF_TEAMS_DOMAIN}'
    )
