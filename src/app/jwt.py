from os import getenv
from jwt import decode
from jwt import PyJWKClient

CF_TEAMS_DOMAIN = getenv('CF_TEAMS_DOMAIN')
CF_JWT_ALGORITHM = "RS256"
CF_ACCESS_CERTS_URL = "https://{}/cdn-cgi/access/certs".format(CF_TEAMS_DOMAIN)

JWT_DECODE_OPTIONS = {
    # Require expiry, issued-at, and email
    "require": ["exp", "iat", "email"],
}


def _get_jwt_signing_key(jwt):
    jwks_client = PyJWKClient(CF_ACCESS_CERTS_URL)
    return jwks_client.get_signing_key_from_jwt(jwt).key


def decode_token(jwt, audience):
    try:
        key = _get_jwt_signing_key(jwt)

        return decode(
            jwt,
            key,
            algorithms=[CF_JWT_ALGORITHM],
            options=JWT_DECODE_OPTIONS,
            audience=audience,
            issuer='https://' + CF_TEAMS_DOMAIN
        )
    except:
        return None
