from flask import request
from werkzeug import Response
from werkzeug.datastructures import Headers
from .jwt import decode_token

def index():
    return Response('use: /auth/<audience>', 404, content_type='text/plain')

def auth(audience):
    jwt = request.headers.get('Cf-Access-Jwt-Assertion')
    
    if jwt is None:
        return Response('no_token', 401, content_type='text/plain')
    
    jwt_data = decode_token(jwt, audience)

    if jwt_data is not None:
        response = Response('', 204)
        response.headers = Headers({'X-Auth-User': jwt_data['email']})

        return response
    
    return Response('invalid_token', 401, content_type='text/plain')
