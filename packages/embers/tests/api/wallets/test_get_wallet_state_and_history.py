import pytest

from tests.client import ApiClient
from tests.conftest import Wallet, assert_match_transfer


def test_get_wallet_state_and_history__empty_wallet(client: ApiClient, wallet: Wallet):
    resp = client.wallets.get_wallet_state_and_history(wallet.address)

    assert resp.status == 200
    assert resp.json["balance"] == "0"
    assert resp.json["requests"] == []
    assert resp.json["exchanges"] == []
    assert resp.json["boosts"] == []
    assert resp.json["transfers"] == []


@pytest.mark.parametrize("funded_wallet", [10_000], indirect=True)
def test_get_wallet_state_and_history__funded_wallet(
    client: ApiClient,
    funded_wallet: Wallet,
    prepopulated_wallet: Wallet,
):
    resp = client.wallets.get_wallet_state_and_history(funded_wallet.address)

    assert resp.status == 200
    assert resp.json["balance"] == "10000"
    assert resp.json["requests"] == []
    assert resp.json["exchanges"] == []
    assert resp.json["boosts"] == []
    assert len(resp.json["transfers"]) == 1
    assert_match_transfer(
        resp.json["transfers"][0],
        {"from": prepopulated_wallet.address, "to": funded_wallet.address, "amount": "10000"},
    )
