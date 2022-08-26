use crate::{application::APP, prelude::*};
use abscissa_core::{clap::Parser, Application, Command, Runnable};
use ethers::signers::Signer;
use gravity_proto::gravity as proto;
use std::time::Duration;

/// Sign delegate keys command
#[derive(Command, Debug, Default, Parser)]
pub struct SignDelegateKeysCmd {
    pub args: Vec<String>,
}

impl Runnable for SignDelegateKeysCmd {
    fn run(&self) {
        let config = APP.config();
        let name = self.args.get(0).expect("ethereum-key-name is required");
        let key = config.load_ethers_wallet(name.clone());

        let val = self.args.get(1).expect("validator-address is required");
        let address = val.parse().expect("Could not parse address");
        abscissa_tokio::run_with_actix(&APP, async {
            let nonce: u64 = match self.args.get(2) {
                Some(nonce) => nonce.parse().expect("cannot parse nonce"),
                None => {
                    let timeout = Duration::from_secs(10);
                    let contact = deep_space::Contact::new(
                        &config.cosmos.grpc,
                        timeout,
                        &config.cosmos.prefix,
                    )
                    .expect("Could not create contact");

                    let account_info = contact.get_account_info(address).await;
                    let account_info = account_info.expect("Did not receive account info");
                    account_info.sequence
                }
            };

            let msg = proto::DelegateKeysSignMsg {
                validator_address: val.clone(),
                nonce,
            };

            let size = prost::Message::encoded_len(&msg);
            let mut buf = bytes::BytesMut::with_capacity(size);
            prost::Message::encode(&msg, &mut buf).expect("Failed to encode DelegateKeysSignMsg!");
            let rt = tokio::runtime::Runtime::new().expect("Could not create tokio runtime");

            let signature = rt
                .block_on(async { key.sign_message(&buf).await })
                .expect("Could not sign message");

            println!("{}", signature);
        })
        .unwrap_or_else(|e| {
            status_err!("executor exited with error: {}", e);
            std::process::exit(1);
        });
    }
}
