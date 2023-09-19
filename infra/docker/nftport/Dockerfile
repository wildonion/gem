FROM python:3.9
RUN pip install fastapi uvicorn requests

RUN USER=root mkdir app

WORKDIR ./app

ADD . ./

WORKDIR /app/core/nftport

EXPOSE 7650


CMD ["uvicorn", "api:app", "--host", "0.0.0.0", "--port", "7650"]