from fastapi import FastAPI
from fastapi.responses import JSONResponse

app = FastAPI()

@app.get("/check-password/{password}")
def check_password(password: str):
    # Read the secret password from the file
    with open("secrets.txt", "r") as file:
        secret_password = file.read().strip()

    # Check if the given password matches the secret password
    if password == secret_password:
        return JSONResponse({"valid": True})
    else:
        return JSONResponse({"valid": False})

if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=2432)