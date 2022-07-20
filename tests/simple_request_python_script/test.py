import requests

abi = [
    {
        "inputs": [
            {"internalType": "address", "name": "recipient", "type": "address"},
            {"internalType": "uint256", "name": "amount", "type": "uint256"},
        ],
        "name": "transfer",
        "outputs": [{"internalType": "bool", "name": "", "type": "bool"}],
        "stateMutability": "nonpayable",
        "type": "function",
    }
]

with open("contract.txt", "r") as f:
    code = f.read()

txn = "0x7b7e9c40f73ec6aa0b14ef61b485d7d41a9b2e70befed0b03face3bf3412c57e"

data = {"txn": txn, "abi": abi, "contract": code}

r = requests.post("http://localhost:8080/", json=data)

print(f"status code: {r.status_code}")
print(f"text: {r.text}")
