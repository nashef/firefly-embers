import base64
from datetime import datetime
from hashlib import blake2b

import pytest
import zbase32
from crc import Calculator, Configuration
from ecdsa import VerifyingKey

from tests.client import Agent, AgentsTeam, ApiClient, Wallet
from tests.key import SECP256k1
from tests.protobuf.rhoapi import ETuple, Expr, Par

ECHO_TEAM = 'context "{}" for A8f132a6a0f9340b59053b2689b8e040b in let A8f132a6a0f9340b59053b2689b8e040b = context "{\\"type\\":\\"input\\"}" for a7e2c4e97cb7145aea7ae4af15ba6789f in let a7e2c4e97cb7145aea7ae4af15ba6789f = < a7e2c4e97cb7145aea7ae4af15ba6789f > in < a7e2c4e97cb7145aea7ae4af15ba6789f > | {context "{\\"type\\":\\"output\\"}" for a02b50756f24f45f793b84b3d33e965c7 in let a02b50756f24f45f793b84b3d33e965c7 = < a02b50756f24f45f793b84b3d33e965c7 > in < a02b50756f24f45f793b84b3d33e965c7 > | 0} in 0 * {(let a7e2c4e97cb7145aea7ae4af15ba6789f = < a7e2c4e97cb7145aea7ae4af15ba6789f > in 0, let a02b50756f24f45f793b84b3d33e965c7 = < a02b50756f24f45f793b84b3d33e965c7 > in 0) * 0} '  # noqa: E501
COMPRESS_TEAM = 'context "{}" for A0fe7f81a4a71433c9b708d8c8484f9b2 in let A0fe7f81a4a71433c9b708d8c8484f9b2 = context "{\\"type\\":\\"input\\"}" for a8f6d37c2166f4aa3936a2df6c65dd9af in let a8f6d37c2166f4aa3936a2df6c65dd9af = < a8f6d37c2166f4aa3936a2df6c65dd9af > in < a8f6d37c2166f4aa3936a2df6c65dd9af > | {context "{\\"type\\":\\"output\\"}" for a72f0d65ffc19427c823558d0224cc964 in let a72f0d65ffc19427c823558d0224cc964 = < a72f0d65ffc19427c823558d0224cc964 > in < a72f0d65ffc19427c823558d0224cc964 > | {context "{\\"type\\":\\"compress\\"}" for a5653f99668994404a9a6ae4bf6889b0e in let a5653f99668994404a9a6ae4bf6889b0e = < a5653f99668994404a9a6ae4bf6889b0e > in < a5653f99668994404a9a6ae4bf6889b0e > | 0}} in 0 * {(let a8f6d37c2166f4aa3936a2df6c65dd9af = < a8f6d37c2166f4aa3936a2df6c65dd9af > in 0, let a5653f99668994404a9a6ae4bf6889b0e = < a5653f99668994404a9a6ae4bf6889b0e > in 0) * {(let a5653f99668994404a9a6ae4bf6889b0e = < a5653f99668994404a9a6ae4bf6889b0e > in 0, let a72f0d65ffc19427c823558d0224cc964 = < a72f0d65ffc19427c823558d0224cc964 > in 0) * 0}} '  # noqa: E501
GPT_COMPRESS_TEAM = 'context "{}" for A005ef27270144757997243d5c7892749 in let A005ef27270144757997243d5c7892749 = context "{\\"type\\":\\"input\\"}" for a2210aecc94d849e38d623251b158a3dd in let a2210aecc94d849e38d623251b158a3dd = < a2210aecc94d849e38d623251b158a3dd > in < a2210aecc94d849e38d623251b158a3dd > | {context "{\\"type\\":\\"output\\"}" for a5822ec2e8030498296ecb192cffc759d in let a5822ec2e8030498296ecb192cffc759d = < a5822ec2e8030498296ecb192cffc759d > in < a5822ec2e8030498296ecb192cffc759d > | {context "{\\"type\\":\\"text-model\\"}" for ad0d3b3ff43b34655807f61eafade165b in let ad0d3b3ff43b34655807f61eafade165b = < ad0d3b3ff43b34655807f61eafade165b > in < ad0d3b3ff43b34655807f61eafade165b > | {context "{\\"type\\":\\"compress\\"}" for a820a9e638bf24771ae6317cf4a9b7d18 in let a820a9e638bf24771ae6317cf4a9b7d18 = < a820a9e638bf24771ae6317cf4a9b7d18 > in < a820a9e638bf24771ae6317cf4a9b7d18 > | 0}}} in 0 * {(let a2210aecc94d849e38d623251b158a3dd = < a2210aecc94d849e38d623251b158a3dd > in 0, let ad0d3b3ff43b34655807f61eafade165b = < ad0d3b3ff43b34655807f61eafade165b > in 0) * {(let ad0d3b3ff43b34655807f61eafade165b = < ad0d3b3ff43b34655807f61eafade165b > in 0, let a820a9e638bf24771ae6317cf4a9b7d18 = < a820a9e638bf24771ae6317cf4a9b7d18 > in 0) * {(let a820a9e638bf24771ae6317cf4a9b7d18 = < a820a9e638bf24771ae6317cf4a9b7d18 > in 0, let a5822ec2e8030498296ecb192cffc759d = < a5822ec2e8030498296ecb192cffc759d > in 0) * 0}}} '  # noqa: E501


@pytest.fixture
def client() -> ApiClient:
    return ApiClient("[::1]:8080")


@pytest.fixture
def prepopulated_wallet(client: ApiClient) -> Wallet:
    wallet = Wallet(key=SECP256k1.from_hex("0B4E12EC24D2F42F3FC826194750E3168A5F03071F382375C29A5E801DBBE8A5"))
    client.wallets.listen_for_deploys(wallet)
    return wallet


@pytest.fixture
def wallet() -> Wallet:
    return Wallet(key=SECP256k1.generate())


@pytest.fixture
def funded_wallet(client: ApiClient, prepopulated_wallet: Wallet, request: pytest.FixtureRequest) -> Wallet:
    wallet = Wallet(key=SECP256k1.generate())
    client.wallets.listen_for_deploys(wallet)
    client.wallets.transfer(from_wallet=prepopulated_wallet, to_wallet=wallet, amount=request.param).wait_for_sync()
    return wallet


@pytest.fixture
def test_wallet(client: ApiClient) -> Wallet:
    resp = client.testnet.test_wallet()
    return Wallet(key=SECP256k1.from_hex(resp.json["key"]))


def assert_match_transfer(transfer: dict, match: dict):
    assert transfer["from"] == match["from"]
    assert transfer["to"] == match["to"]
    assert transfer["amount"] == match["amount"]
    assert transfer.get("description") == match.get("description")


@pytest.fixture
def agent(client: ApiClient, funded_wallet: Wallet, request: pytest.FixtureRequest) -> Agent:
    resp = client.ai_agents.create(
        funded_wallet,
        name="my_agent",
        code='@Nil!("foo")' if not hasattr(request, "param") else request.param,
    ).wait_for_sync()

    return Agent(
        id=resp.first.json["id"],
        version=resp.first.json["version"],
        name="my_agent",
        code='@Nil!("foo")',
    )


def assert_match_agent_header(header: dict, match: Agent):
    assert header["id"] == match.id
    assert header["version"] == match.version
    assert header.get("created_at")
    assert header["name"] == match.name
    assert header.get("shard") == match.shard
    assert header.get("logo") == match.logo


def assert_match_agent(agent: dict, match: Agent):
    assert agent["id"] == match.id
    assert agent["version"] == match.version
    assert agent.get("created_at")
    assert agent["name"] == match.name
    assert agent.get("shard") == match.shard
    assert agent.get("logo") == match.logo
    assert agent.get("code") == match.code


@pytest.fixture
def agents_team(client: ApiClient, funded_wallet: Wallet, request: pytest.FixtureRequest) -> AgentsTeam:
    resp = client.ai_agents_teams.create(
        funded_wallet,
        name="my_agents_team",
        graph="< foo > | 0 " if not hasattr(request, "param") else request.param,
    ).wait_for_sync()

    return AgentsTeam(
        id=resp.first.json["id"],
        version=resp.first.json["version"],
        name="my_agents_team",
        graph="< foo > | 0 ",
    )


def assert_match_agents_team_header(header: dict, match: AgentsTeam):
    assert header["id"] == match.id
    assert header["version"] == match.version
    assert header.get("created_at")
    assert header["name"] == match.name
    assert header.get("shard") == match.shard
    assert header.get("logo") == match.logo


def assert_match_agents_team(team: dict, match: AgentsTeam):
    assert team["id"] == match.id
    assert team["version"] == match.version
    assert team.get("created_at")
    assert team["name"] == match.name
    assert team.get("shard") == match.shard
    assert team.get("logo") == match.logo
    assert team.get("graph") == match.graph


def insert_signed_signature(key: SECP256k1, timestamp: int, wallet: Wallet, version: int) -> bytes:
    to_sign = bytes(
        Par(
            exprs=[
                Expr(
                    e_tuple_body=ETuple(
                        ps=[
                            Par(exprs=[Expr(g_int=timestamp)]),
                            Par(exprs=[Expr(g_byte_array=wallet.key.public_key_bytes)]),
                            Par(exprs=[Expr(g_int=version)]),
                        ],
                    ),
                ),
            ],
        ),
    )

    return key.sign(to_sign)


def insert_signed_deploy(key: SECP256k1, timestamp: datetime, wallet: Wallet, version: int) -> dict:
    millis = int(timestamp.timestamp() * 1000)
    signature = insert_signed_signature(key, millis, wallet, version)

    return {
        "timestamp": str(millis),
        "version": str(version),
        "uri_pub_key": key.public_key_bytes.hex(),
        "signature": base64.b64encode(signature).decode(),
    }


def public_key_to_uri(public_key: VerifyingKey) -> str:
    hash_bytes = blake2b(public_key.to_string("uncompressed"), digest_size=32).digest()

    crc_config = Configuration(
        width=14,
        polynomial=0x4805,
        init_value=0x0000,
        final_xor_value=0x0000,
        reverse_input=False,
        reverse_output=False,
    )

    crc14 = Calculator(crc_config, optimized=False)
    crc_val = crc14.checksum(hash_bytes)
    crc_le = crc_val.to_bytes(2, "little")

    full_key = hash_bytes + bytes([crc_le[0]]) + bytes([(crc_le[1] << 2) & 0xFF])
    encoded = zbase32.encode(full_key)[: 270 // 5]

    return f"rho:id:{encoded}"
