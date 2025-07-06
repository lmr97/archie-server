# largely just copied from Python's official Docker image page
FROM python:3.13-slim

RUN useradd -m converter
USER converter
WORKDIR /home/converter

COPY --chown=converter:converter letterboxd_get_list ./
COPY --chown=converter:converter lb_app.py           ./
COPY --chown=converter:converter healthcheck.py      ./

RUN pip install ./letterboxd_list

# -u option is for unbuffered output, so print statements will show up
# in the the Docker container's logs
HEALTHCHECK \
    --interval=30s \
    --timeout=5s \
    --start-period=2s \
    --retries=3 \
    CMD [ "python3", "-u", "healthcheck.py" ]

EXPOSE 3575
CMD [ "python3", "-u", "lb_app.py" ]