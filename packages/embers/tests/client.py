from __future__ import annotations

import base64
import json
import threading
from dataclasses import dataclass
from functools import cached_property
from hashlib import blake2b
from typing import Any, Self

import base58
import requests
import websocket
from Crypto.Hash import keccak

from tests.key import SECP256k1

DEFAULT_TIMEOUT = 15

FIRECAP_ID = bytes([0, 0, 0])
FIRECAP_VERSION = bytes([0])


class ApiSync:
    def __init__(self):
        self._lock = threading.Lock()
        self._seen = set()
        self._waiting = {}

    def register(self, deploy_id: str) -> threading.Event:
        event = threading.Event()
        with self._lock:
            if deploy_id in self._seen:
                event.set()
                return event
            self._waiting[deploy_id] = event
            return event

    def notify(self, deploy_id: str):
        with self._lock:
            self._seen.add(deploy_id)
            event = self._waiting.get(deploy_id)
            if event is not None:
                event.set()


@dataclass
class Responce:
    status: int
    body: str

    def __init__(self, r: requests.Response):
        self.status = r.status_code
        self.body = r.text

    @cached_property
    def json(self) -> Any:
        return json.loads(self.body)


class UpdateResponce:
    def __init__(self, first: Responce, second: Responce, accepted: threading.Event):
        self.first = first
        self.second = second
        self._accepted = accepted

    def wait_for_sync(self) -> Self:
        assert self._accepted.wait(timeout=DEFAULT_TIMEOUT)
        return self


class HttpClient:
    def __init__(self, base_url: str):
        self.base_url = base_url
        self.listeners: dict[str, ApiSync] = {}

    def get(self, url: str, timeout: int = DEFAULT_TIMEOUT) -> Responce:
        url = f"http://{self.base_url}/api/{url}"
        r = requests.get(url, timeout=timeout)
        return Responce(r)

    def post(self, url: str, json: Any | None = None, timeout: int = DEFAULT_TIMEOUT) -> Responce:
        url = f"http://{self.base_url}/api/{url}"
        r = requests.post(url, json=json, timeout=timeout)
        return Responce(r)


class TestnetApi:
    def __init__(self, client: HttpClient):
        self._client = client

    def test_wallet(self) -> Responce:
        resp = self._client.post("/testnet/wallet")
        assert resp.status == 200
        return resp

    def deploy(self, wallet: Wallet, test: str, env: str | None = None) -> Responce:
        resp = self._client.post("/testnet/deploy/prepare", json={"test": test, "env": env})
        assert resp.status == 200

        json = {
            "test": sing_contract(wallet, resp.json["test_contract"]),
            "env": sing_contract(wallet, resp.json["env_contract"])
            if resp.json.get("env_contract") is not None
            else None,
        }

        resp_next = self._client.post("/testnet/deploy/send", json=json)
        assert resp_next.status == 200

        return resp_next


@dataclass
class Wallet:
    key: SECP256k1

    @cached_property
    def address(self) -> str:
        key_hash = keccak.new(digest_bits=256).update(self.key.public_key_bytes[1:]).digest()
        eth_hash = keccak.new(digest_bits=256).update(key_hash[-20:]).digest()

        checksum_hash = blake2b(FIRECAP_ID + FIRECAP_VERSION + eth_hash, digest_size=32).digest()
        checksum = checksum_hash[:4]

        return base58.b58encode(FIRECAP_ID + FIRECAP_VERSION + eth_hash + checksum).decode()


class WalletsApi:
    def __init__(self, client: HttpClient):
        self._client = client

    def get_wallet_state_and_history(self, address: str) -> Responce:
        return self._client.get(f"/wallets/{address}/state")

    def transfer(
        self,
        from_wallet: Wallet,
        to_wallet: Wallet,
        amount: int,
        description: str | None = None,
    ) -> UpdateResponce:
        resp = self._client.post(
            "/wallets/transfer/prepare",
            json={"from": from_wallet.address, "to": to_wallet.address, "amount": amount, "description": description},
        )
        assert resp.status == 200

        resp_next = self._client.post("/wallets/transfer/send", json=sing_contract(from_wallet, resp.json["contract"]))
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[from_wallet.address].register(resp_next.json["deploy_id"]),
        )

    def boost(
        self,
        from_wallet: Wallet,
        to_wallet: Wallet,
        amount: int,
        post_author_did: str,
        description: str | None = None,
        post_id: str | None = None,
    ) -> UpdateResponce:
        resp = self._client.post(
            "/wallets/boost/prepare",
            json={
                "from": from_wallet.address,
                "to": to_wallet.address,
                "amount": amount,
                "description": description,
                "post_author_did": post_author_did,
                "post_id": post_id,
            },
        )
        assert resp.status == 200

        resp_next = self._client.post("/wallets/boost/send", json=sing_contract(from_wallet, resp.json["contract"]))
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[from_wallet.address].register(resp_next.json["deploy_id"]),
        )

    def listen_for_deploys(self, wallet: Wallet):
        api_sync = ApiSync()

        def on_message(_: Any, msg: str):
            event = json.loads(msg)
            if event.get("node_type") == "Observer":
                api_sync.notify(event["deploy_id"])

        ws = websocket.WebSocketApp(
            url=f"ws://{self._client.base_url}/api/wallets/{wallet.address}/deploys",
            on_message=on_message,
        )

        thread = threading.Thread(target=ws.run_forever, daemon=True)
        thread.start()

        self._client.listeners[wallet.address] = api_sync


@dataclass
class Agent:
    id: str
    version: str
    name: str
    description: str | None = None
    shard: str | None = None
    logo: str | None = None
    code: str | None = None


class AiAgentsApi:
    def __init__(self, client: HttpClient):
        self._client = client

    def list(self, address: str) -> Responce:
        return self._client.get(f"/ai-agents/{address}")

    def list_versions(self, address: str, agent_id: str) -> Responce:
        return self._client.get(f"/ai-agents/{address}/{agent_id}/versions")

    def get(self, address: str, agent_id: str, agent_version: str) -> Responce:
        return self._client.get(f"/ai-agents/{address}/{agent_id}/versions/{agent_version}")

    def create(
        self,
        wallet: Wallet,
        name: str,
        description: str | None = None,
        shard: str | None = None,
        logo: str | None = None,
        code: str | None = None,
    ) -> UpdateResponce:
        resp = self._client.post(
            "/ai-agents/create/prepare",
            json={"name": name, "description": description, "shard": shard, "logo": logo, "code": code},
        )
        assert resp.status == 200

        resp_next = self._client.post("/ai-agents/create/send", json=sing_contract(wallet, resp.json["contract"]))
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[wallet.address].register(resp_next.json["deploy_id"]),
        )

    def save(
        self,
        wallet: Wallet,
        agent_id: str,
        name: str,
        description: str | None = None,
        shard: str | None = None,
        logo: str | None = None,
        code: str | None = None,
    ) -> UpdateResponce:
        resp = self._client.post(
            f"/ai-agents/{agent_id}/save/prepare",
            json={"name": name, "description": description, "shard": shard, "logo": logo, "code": code},
        )
        assert resp.status == 200

        resp_next = self._client.post(
            f"/ai-agents/{agent_id}/save/send",
            json=sing_contract(wallet, resp.json["contract"]),
        )
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[wallet.address].register(resp_next.json["deploy_id"]),
        )

    def delete(self, wallet: Wallet, agent_id: str) -> UpdateResponce:
        resp = self._client.post(f"/ai-agents/{agent_id}/delete/prepare")
        assert resp.status == 200

        resp_next = self._client.post(
            f"/ai-agents/{agent_id}/save/send",
            json=sing_contract(wallet, resp.json["contract"]),
        )
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[wallet.address].register(resp_next.json["deploy_id"]),
        )


@dataclass
class AgentsTeam:
    id: str
    version: str
    name: str
    description: str | None = None
    shard: str | None = None
    logo: str | None = None
    graph: str | None = None


class AiAgentsTeamsApi:
    def __init__(self, client: HttpClient):
        self._client = client

    def list(self, address: str) -> Responce:
        return self._client.get(f"/ai-agents-teams/{address}")

    def list_versions(self, address: str, agent_id: str) -> Responce:
        return self._client.get(f"/ai-agents-teams/{address}/{agent_id}/versions")

    def get(self, address: str, agent_id: str, agent_version: str) -> Responce:
        return self._client.get(f"/ai-agents-teams/{address}/{agent_id}/versions/{agent_version}")

    def create(
        self,
        wallet: Wallet,
        name: str,
        description: str | None = None,
        shard: str | None = None,
        logo: str | None = None,
        graph: str | None = None,
    ) -> UpdateResponce:
        resp = self._client.post(
            "/ai-agents-teams/create/prepare",
            json={"name": name, "description": description, "shard": shard, "logo": logo, "graph": graph},
        )
        assert resp.status == 200

        resp_next = self._client.post("/ai-agents-teams/create/send", json=sing_contract(wallet, resp.json["contract"]))
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[wallet.address].register(resp_next.json["deploy_id"]),
        )

    def deploy(self, wallet: Wallet, agents_team: AgentsTeam, phlo_limit: int, deploy: dict) -> UpdateResponce:
        resp = self._client.post(
            "/ai-agents-teams/deploy/prepare",
            json={
                "type": "AgentsTeam",
                "id": agents_team.id,
                "version": agents_team.version,
                "address": wallet.address,
                "phlo_limit": phlo_limit,
                "deploy": deploy,
            },
        )
        assert resp.status == 200

        resp_next = self._client.post(
            "/ai-agents-teams/deploy/send",
            json={
                "contract": sing_contract(wallet, resp.json["contract"]),
                "system": sing_contract(wallet, resp.json["system"]) if resp.json.get("system") is not None else None,
            },
        )
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[wallet.address].register(resp_next.json["deploy_id"]),
        )

    def deploy_graph(self, wallet: Wallet, graph: str, phlo_limit: int, deploy: dict) -> UpdateResponce:
        resp = self._client.post(
            "/ai-agents-teams/deploy/prepare",
            json={"type": "Graph", "graph": graph, "phlo_limit": phlo_limit, "deploy": deploy},
        )
        assert resp.status == 200

        resp_next = self._client.post(
            "/ai-agents-teams/deploy/send",
            json={
                "contract": sing_contract(wallet, resp.json["contract"]),
                "system": sing_contract(wallet, resp.json["system"]) if resp.json.get("system") is not None else None,
            },
        )
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[wallet.address].register(resp_next.json["deploy_id"]),
        )

    def run(self, wallet: Wallet, prompt: str, phlo_limit: int, agents_team: str) -> Responce:
        resp = self._client.post(
            "/ai-agents-teams/run/prepare",
            json={"prompt": prompt, "phlo_limit": phlo_limit, "agents_team": agents_team},
        )
        assert resp.status == 200

        resp_next = self._client.post("/ai-agents-teams/run/send", json=sing_contract(wallet, resp.json["contract"]))
        assert resp_next.status == 200

        return resp_next

    def save(
        self,
        wallet: Wallet,
        agent_id: str,
        name: str,
        description: str | None = None,
        shard: str | None = None,
        logo: str | None = None,
        graph: str | None = None,
    ) -> UpdateResponce:
        resp = self._client.post(
            f"/ai-agents-teams/{agent_id}/save/prepare",
            json={"name": name, "description": description, "shard": shard, "logo": logo, "graph": graph},
        )
        assert resp.status == 200

        resp_next = self._client.post(
            f"/ai-agents-teams/{agent_id}/save/send",
            json=sing_contract(wallet, resp.json["contract"]),
        )
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[wallet.address].register(resp_next.json["deploy_id"]),
        )

    def delete(self, wallet: Wallet, agent_id: str) -> UpdateResponce:
        resp = self._client.post(f"/ai-agents-teams/{agent_id}/delete/prepare")
        assert resp.status == 200

        resp_next = self._client.post(
            f"/ai-agents-teams/{agent_id}/save/send",
            json=sing_contract(wallet, resp.json["contract"]),
        )
        assert resp_next.status == 200

        return UpdateResponce(
            first=resp,
            second=resp_next,
            accepted=self._client.listeners[wallet.address].register(resp_next.json["deploy_id"]),
        )


class ApiClient:
    def __init__(self, backend_url: str):
        self._http_client = HttpClient(backend_url)
        self.testnet = TestnetApi(self._http_client)
        self.wallets = WalletsApi(self._http_client)
        self.ai_agents = AiAgentsApi(self._http_client)
        self.ai_agents_teams = AiAgentsTeamsApi(self._http_client)


def sing_contract(wallet: Wallet, contract: Any) -> dict:
    signature = wallet.key.sign(base64.b64decode(contract, validate=True))

    return {
        "contract": contract,
        "sig_algorithm": "secp256k1",
        "sig": base64.b64encode(signature).decode(),
        "deployer": base64.b64encode(wallet.key.public_key_bytes).decode(),
    }
