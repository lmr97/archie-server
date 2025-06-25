# largely just copied from Python's official Docker image page
FROM python:3.13-slim

RUN useradd -m converter
USER converter
WORKDIR /home/converter

COPY --chown=converter:converter letterboxd_get_list ./
COPY --chown=converter:converter lb_app.py           ./
COPY --chown=converter:converter healthcheck.py      ./

RUN pip install ./letterboxd_list
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD [ "python3", "healthcheck.py" ]
CMD [ "python3", "lb_app.py" ]