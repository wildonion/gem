


a simple check password server using fastapi

> make sure you have an updated password inside the `secrets.txt`

## Usage

```bash
pip3 install fastapi && pip3 install "uvicorn[standard]"
pm2 start "uvicorn main:app --host 0.0.0.0 --port 2432" --name checkpswd
```