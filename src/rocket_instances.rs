// First code snippet
use std::sync::{Arc, Mutex};
use rocket::Build;
use rocket::config::{TlsConfig, MutualTls};

use crate::endpoints::pub_endpoints::{key_gen, sign, verify, receive_broadcast};

use crate::communication::create_communication_channel;

use crate::mpc_config::MPCconfig;

pub struct ServerConfigState {
    config: Mutex<MPCconfig>,
}

impl ServerConfigState {
    pub fn config(&self) -> &Mutex<MPCconfig> {
        &self.config
    }
}

pub struct SharedDb(pub Arc<create_communication_channel::Db>);

impl Clone for SharedDb {
    fn clone(&self) -> Self {
        SharedDb(self.0.clone())
    }
}

impl std::ops::Deref for SharedDb {
    type Target = create_communication_channel::Db;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn rocket_with_client_auth(
    figment: rocket::figment::Figment,
    config: MPCconfig,
    db: SharedDb,
    port: u16,
) -> rocket::Rocket<Build> {
    let public_cert = format!("certs/public/cert_{}.pem", config.server_id());
    let private_cert = format!("certs/private/private_key_{}.pem", config.server_id());

    let tls_config = TlsConfig::from_paths(public_cert, private_cert)
        .with_mutual(MutualTls::from_path("certs/ca_cert.pem").mandatory(true))
    ;


// Create a figment with the desired configuration
    let figment = figment
        .merge(("tls", tls_config))
        .merge(("port", port));

    rocket::custom(figment)
        .mount("/", rocket::routes![receive_broadcast])
        .manage(ServerConfigState { config: Mutex::new(config)})
        .manage(db)
}

pub fn rocket_without_client_auth(
    figment: rocket::figment::Figment,
    config: MPCconfig,
    db: SharedDb,
    port: u16
) -> rocket::Rocket<Build> {
    let figment = figment.merge(("port", port));

    rocket::custom(figment)
        .mount("/",
               rocket::routes![key_gen, sign, verify])
        .manage(ServerConfigState { config: Mutex::new(config)})
        .manage(db)
}