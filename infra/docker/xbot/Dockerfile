

FROM python:3.9-slim

WORKDIR /app

ADD . ./

WORKDIR /app/core/xbot

RUN apt-get update \
    && apt-get -y install libpq-dev gcc \
    && pip install psycopg2

RUN pip install --no-cache-dir -r requirements.txt

COPY . .

CMD ["uvicorn", "bot:app", "--host", "0.0.0.0", "--port", "4245", "--reload"]