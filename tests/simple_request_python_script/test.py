import requests
import json

with open("abi.json", "r") as f:
    abi = json.load(f)


with open("contract.txt", "r") as f:
    code = f.read()

txn = "0x7b7e9c40f73ec6aa0b14ef61b485d7d41a9b2e70befed0b03face3bf3412c57e"

data = {"txn": txn, "abi": abi, "contract": code}

r = requests.post("http://localhost:8080/", json=data)

print(f"status code: {r.status_code}")
print(f"text: {r.text}")
