from flask import Flask
from . import views

app = Flask(__name__)

@app.errorhandler(404)
def page_not_found(e):
    return '', 404

app.add_url_rule('/', view_func=views.index)
app.add_url_rule('/auth/<audience>', view_func=views.auth)
