use rustls::*;
use serde_json::from_str;
use serde_json::json;
use serde_json::Value;
use tungstenite::{connect, Message};
use url::Url;

// use electrum_client::Call
#[tauri::command]
pub fn ws_subscribe(address: &str, network_url: &str) -> Result<Value, ()> {
    let mut subscription: Value = "".into();
    //use ws://
    println!("HELLO {:?} {:?}", network_url, address);
    if let Ok(url) = Url::parse(network_url) {
        println!("HELLO2");
        if let Ok((mut socket, response)) = connect(url.clone()) {
            println!("Connected to the server");
            println!("Response HTTP code: {}", response.status());
            println!("Response contains the following headers:");
            for (ref header, _value) in response.headers() {
                println!("* {}", header);
            }
            let value = json!({
                "method": "blockchain.address.subscribe",
                "params":   [
                    address
                ],
                "id":1
            });

            socket.send(Message::Text(value.to_string()));

            loop {
                let msg = socket.read();
                match msg.as_ref() {
                    Ok(res) => match res {
                        Message::Text(res) => {
                            match from_str::<Value>(res) {
                                Ok(scripthash) => match scripthash {
                                    Value::Object(data) => {
                                        // println!("DATA {:?}", data);
                                        subscription = data["result"].clone();
                                        println!("SUSCRIBED {:?}", subscription);
                                        // subscription
                                    }
                                    _ => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Message::Ping(msg) => {
                            println!("Connection is alive {:?}", msg)
                        }
                        _ => {}
                    },
                    Err(e) => {}
                }
                println!("Received: {:?}", msg);
            }
            // socket.close(None);
        };
        println!("{:?}", connect(url));
    }
    Ok(subscription.clone())
}

mod test {
    use super::*;
    #[test]
    fn maain() {
        // env_logger::init();

        let (mut socket, response) =
            connect(Url::parse("ws://localhost:50003").unwrap()).expect("Can't connect");

        println!("Connected to the server");
        println!("Response HTTP code: {}", response.status());
        println!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            println!("* {}", header);
        }
        let request = r#"{"jsonrpc": "2.0", "method": "blockchain.address.subscribe", "params": [bchtest:zzxu4ynqdgyjr2hvt5xcx7x35ncdz8zffswqykaryj], "id": 1}"#;
        socket.send(Message::Text(request.into())).unwrap();
        loop {
            let msg = socket.read().expect("Error reading message");
            println!("Received: {}", msg);
        }
        // socket.close(None);
    }
}
