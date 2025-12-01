from datetime import UTC, datetime

import pytest

from tests.client import AgentsTeam, ApiClient, Wallet
from tests.conftest import COMPRESS_TEAM, ECHO_TEAM, GPT_COMPRESS_TEAM, insert_signed_deploy
from tests.key import SECP256k1


@pytest.mark.parametrize("funded_wallet", [100_000_000], indirect=True)
@pytest.mark.parametrize("graph", [ECHO_TEAM, COMPRESS_TEAM, GPT_COMPRESS_TEAM])
def test_deploy_graph(client: ApiClient, funded_wallet: Wallet, graph: str):
    deploy = insert_signed_deploy(
        SECP256k1.generate(),
        datetime.now(UTC),
        funded_wallet,
        version=0,
    )

    resp = client.ai_agents_teams.deploy_graph(
        funded_wallet,
        graph=graph,
        phlo_limit=5_000_000,
        deploy=deploy,
    )
    assert resp.first.status == 200


@pytest.mark.parametrize("funded_wallet", [100_000_000], indirect=True)
@pytest.mark.parametrize("agents_team", [ECHO_TEAM, COMPRESS_TEAM, GPT_COMPRESS_TEAM], indirect=True)
def test_deploy_agents_team(client: ApiClient, funded_wallet: Wallet, agents_team: AgentsTeam):
    deploy = insert_signed_deploy(
        SECP256k1.generate(),
        datetime.now(UTC),
        funded_wallet,
        version=0,
    )

    resp = client.ai_agents_teams.deploy(
        funded_wallet,
        agents_team=agents_team,
        phlo_limit=5_000_000,
        deploy=deploy,
    ).wait_for_sync()
    assert resp.first.status == 200

    resp = client.ai_agents_teams.get(funded_wallet.address, agents_team.id, agents_team.version)
    assert resp.json["last_deploy"]
    assert resp.json["uri"]

    resp = client.ai_agents_teams.list_versions(funded_wallet.address, agents_team.id)
    assert len(resp.json["agents_teams"]) == 1
    assert resp.json["agents_teams"][0]["last_deploy"]

    resp = client.ai_agents_teams.list(funded_wallet.address)
    assert len(resp.json["agents_teams"]) == 1
    assert resp.json["agents_teams"][0]["last_deploy"]
