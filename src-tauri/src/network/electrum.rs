use bitcoinsuite_core::{
    hash::{Hashed, Sha256},
    script::Script,
};
use electrum_client::{Client, ElectrumApi, Param};

// /* type SerError = */ Box<dyn std::error::Error>

// #[tauri::command]
pub async fn get_unspent_utxos(
    address: &str,
    network_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("ELECTRUM REQUEST UNSPENT UTXOS: \n==>{}<==\n:", address);
    let client = Client::new(network_url)?;
    let method = "blockchain.address.listunspent";
    let params = vec![
        Param::String(address.to_string()),
        Param::String("include_tokens".to_string()),
    ];
    let res = ElectrumApi::raw_call(&client, &method, params);
    let utxo_data = serde_json::to_string(&res?.as_array());

    Ok(utxo_data?)
}

// #[tauri::command]
pub async fn get_utxos_balance_no_tokens(
    address: &str,
    network_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("ELECTRUM REQUEST ADDRESS BALANCE: \n==> {} <==\n", address);
    let client = Client::new(network_url)?;
    let method = "blockchain.address.get_balance";
    let params = vec![Param::String(address.to_string())];
    let res = ElectrumApi::raw_call(&client, &method, params);
    let balance = serde_json::to_string(&res?.as_object());
    Ok(balance?)
}

// #[tauri::command]
pub async fn get_utxos_balance_include_tokens(
    address: &str,
    network_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("ELECTRUM REQUEST ADDRESS BALANCE: \n==>{}<==\n:", address);
    let client = Client::new(network_url)?;
    let method = "blockchain.address.get_balance";
    let params = vec![
        Param::String(address.to_string()),
        Param::String("include_tokens".to_string()),
    ];
    let res = ElectrumApi::raw_call(&client, &method, params);
    let balance = serde_json::to_string(&res?.as_object());
    Ok(balance?)
}

// #[tauri::command]
pub async fn get_unspent_non_token_utxos(
    address: &str,
    network_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new(network_url)?;
    let method = "blockchain.address.listunspent";
    let params = vec![Param::String(address.to_string())];
    let res = ElectrumApi::raw_call(&client, &method, params);
    let utxo_data = serde_json::to_string(&res?.as_array());

    Ok(utxo_data?)
}

// #[tauri::command]
pub async fn get_address_history(
    address: &str,
    network_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new(network_url)?;
    let method = "blockchain.address.get_history";

    let params = vec![Param::String(address.to_string())];
    let res = ElectrumApi::raw_call(&client, &method, params);
    let addr_history = serde_json::to_string(&res?.as_array());

    Ok(addr_history?)
}

fn to_electrum_scripthash(script: &Script) -> [u8; 32] {
    Sha256::digest(script).to_le_bytes()
}

// #[tauri::command]
pub async fn subscribe(
    address: &str,
    network_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new(network_url)?;
    let method = "blockchain.address.subscribe";
    let params = vec![Param::String(address.to_string())];
    let res = ElectrumApi::raw_call(&client, &method, params);

    match res {
        Ok(res) => {
            println!("subscribe hash{:?}", res);
            Ok(res.as_str().unwrap().to_string())
        }
        Err(e) => Err(e.into()),
    }
}

// #[tauri::command]
pub async fn get_mempool(
    address: &str,
    network_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new(network_url)?;
    let method = "blockchain.address.get_mempool";
    let params = vec![Param::String(address.to_string())];
    let res = ElectrumApi::raw_call(&client, &method, params);

    match res {
        Ok(res) => {
            println!("get_mempool {:?}", res);
            Ok(res.to_string())
        }
        Err(e) => Err(e.into()),
    }
}

#[tauri::command]
pub async fn unsubscribe(
    address: &str,
    network_url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let client = Client::new(network_url)?;
    let method = "blockchain.address.unsubscribe";
    let params = vec![Param::String(address.to_string())];
    let res = ElectrumApi::raw_call(&client, &method, params);
    match res {
        Ok(res) => {
            println!("unsubscribed: {:?}", res);
            Ok(res.as_bool().unwrap())
        }
        Err(e) => Err(e.into()),
    }
}

#[tauri::command]
pub async fn ping(network: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new(network)?;
    let method = "server.ping";
    let params = vec![];
    let res = ElectrumApi::raw_call(&client, &method, params);
    println!("PING {:?}", res);
    let res = serde_json::to_string(&res?.as_array());
    Ok(res?)
}
#[tauri::command]
pub async fn send_raw_transaction(
    transaction: &str,
    network_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Broadcasting Transaction");
    let client = Client::new(network_url)?;
    let method = "blockchain.transaction.broadcast";
    let params = vec![Param::String(transaction.to_string())];
    let res = ElectrumApi::raw_call(&client, &method, params);
    let res = serde_json::to_string(&res?.as_str());
    println!("TXID {:?}", res);
    Ok(res?)
}

#[allow(dead_code)]
mod test {
    use bitcoinsuite_core::hash::{Hashed, ShaRmd160};
    use electrum_client::{
        bitcoin::Script,
        raw_client::{self, RawClient},
        ElectrumApi, Param,
    };
    use std::str::FromStr;

    fn to_electrum_scripthash(script: &Script) -> [u8; 32] {
        Sha256::digest(script).to_le_bytes()
    }

    fn get_test_server() -> String {
        // "tcp://chipnet.imaginary.cash:50001".to_string()
        "0.0.0.0:50001".to_string()
        // std::env::var("TEST_ELECTRUM_SERVER").unwrap_or("electrum.blockstream.info:50001".into())
    }
    use crate::address::address_to_pubkey_hash;

    use super::*;

    #[allow(dead_code)]
    #[tokio::test]
    async fn test_api() {
        let res = get_unspent_non_token_utxos(
            "bchtest:qptnz3u8atavszhaqk037v0fjrtahxmsl5mm45u3pf",
            // "tcp://localhost:50001",
            "ws://localhost:50003",
            // "tcp://chipnet.imaginary.cash:50001",
        );
        println!("{:?}", res.await);
    }
    #[test]
    fn test_script_subscribe() {
        use bitcoincash_addr::Address;
        use bitcoinsuite_core::script::Script as bScript;
        use std::str::FromStr;

        // let client = Client::new(get_test_server().as_str()).unwrap();
        // let client = Client::new(get_test_server().as_str()).unwrap();
        let client = RawClient::new(get_test_server(), None).unwrap();
        let script_hash =
            // address_to_pubkey_hash("bchtest:qptnz3u8atavszhaqk037v0fjrtahxmsl5mm45u3pf").unwrap();
            address_to_pubkey_hash("bchtest:zzxu4ynqdgyjr2hvt5xcx7x35ncdz8zffswqykaryj").unwrap();
        // let script_hash = to_electrum_scripthash(Script::from_bytes(&script_hash));

        let script_hash = hex::encode::<&[u8]>(script_hash.as_ref());
        let script_hash = ShaRmd160::from_le_hex(&script_hash);
        let p2pkh = bScript::p2pkh(&script_hash.unwrap());

        let x = client
            .script_get_balance(&Script::from_bytes(p2pkh.bytecode()))
            // .script_subscribe(&script_hash)
            .unwrap();
        let y = client.script_unsubscribe(&Script::from_bytes(p2pkh.bytecode()));
        // .script_subscribe(&script_hash)
        // .unwrap();

        println!("{:?}", x);
        println!("{:?}", y);
    }
    #[test]
    fn test_ping() {
        let client = Client::new(get_test_server().as_str()).unwrap().ping();
        assert_eq!(client.is_ok(), true);
    }
}
