#######################################
# Base image with defaults
#

FROM python:3.8-slim AS base

ARG APP_USER=app
ARG APP_VIRTUALENV=/opt/venv

# set environment variables
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8
ENV PYTHONUNBUFFERED=1
ENV PYTHONFAULTHANDLER=1
ENV PYTHONDONTWRITEBYTECODE=1
ENV APP_VIRTUALENV=${APP_VIRTUALENV}

RUN pip install --upgrade pip

#######################################
# Install python dependencies
#

FROM base as py-dependencies

# install system dependencies
RUN apt-get update && \
	apt-get install -y --no-install-recommends gcc rustc && \
	apt-get autoremove -y && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists/*

RUN --security=insecure \
	mkdir -p /root/.cargo && \
	chmod 777 /root/.cargo && \
	mount -t tmpfs none /root/.cargo && \
	pip3 install --no-cache-dir cryptography

# create new virtualenv
RUN python -m venv $APP_VIRTUALENV

# use the virtualenv
ENV PATH="$APP_VIRTUALENV/bin:$PATH"

# install python dependencies
COPY requirements.txt .
RUN pip install -r requirements.txt --no-cache-dir

#######################################
# Build image for runtime
#
FROM base as runtime

# copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

# create less privileged user
RUN useradd $APP_USER && \
	chmod +x /usr/local/bin/docker-entrypoint.sh

# copy python dependencies
COPY --from=py-dependencies $APP_VIRTUALENV $APP_VIRTUALENV

# use the virtualvenv
ENV PATH="$APP_VIRTUALENV/bin:$PATH"

WORKDIR /usr/src/app

COPY --chown=$APP_USER:$APP_USER src .
RUN chown -R $APP_USER:$APP_USER .

# don't run as root!
USER $APP_USER

# gunicorn default port
EXPOSE 8000

ENTRYPOINT [ "/usr/local/bin/docker-entrypoint.sh" ]

CMD [ "gunicorn", "--worker-tmp-dir=/dev/shm", "--workers=2", "--threads=4", "--worker-class=gthread", "--bind=0.0.0.0", "--log-file=-", "app:app"]
