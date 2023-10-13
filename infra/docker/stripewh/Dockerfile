
FROM python:3.9-slim

WORKDIR /app

ADD . ./

COPY ./.env /app/core/stripewh/.env

WORKDIR /app/core/stripewh

RUN apt-get update \
    && apt-get -y install libpq-dev gcc \
    && pip install psycopg2
# Install any dependencies
RUN pip install --no-cache-dir -r requirements.txt

ENV FLASK_APP=server.py

# Specify the command to run on container start
CMD ["python3", "-m", "flask", "run", "--host=0.0.0.0", "--port=4242"]