// First code snippet
use std::sync::{Arc, Mutex};
use rocket::Build;
use crate::{
    create_communication_channel::Db,
    create_communication_channel::receive_broadcast,
    key_gen,
    sign,
};



pub struct ServerIdState{
    pub server_id: Mutex<u16>,
}

pub struct SharedDb(pub Arc<Db>);

impl Clone for SharedDb {
    fn clone(&self) -> Self {
        SharedDb(self.0.clone())
    }
}

impl std::ops::Deref for SharedDb {
    type Target = Db;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn rocket_with_client_auth(
    figment: rocket::figment::Figment,
    server_id: u16,
    db: SharedDb,
    port: u16,
) -> rocket::Rocket<Build> {
    // let tls_config = TlsConfig {
    //     certs: "path/to/your/certificate.pem".into(),
    //     key: "path/to/your/private/key.pem".into(),
    //     ca: "path/to/your/ca_certificate.pem".into(),
    //     client_auth: ClientAuth::Required,
    // };

// Create a figment with the desired configuration
    let figment = figment
        // .merge(("tls", tls_config))
        .merge(("port", port));

    rocket::custom(figment)
        .mount("/", rocket::routes![receive_broadcast])
        .manage(ServerIdState{server_id: Mutex::new(server_id)})
        .manage(db)
}

pub fn rocket_without_client_auth(
    figment: rocket::figment::Figment,
    server_id: u16,
    db: SharedDb,
    port: u16,
) -> rocket::Rocket<Build> {
    let figment = figment.merge(("port", port));

    rocket::custom(figment)
        .mount("/", rocket::routes![key_gen, sign, verify])
        .manage(ServerIdState{server_id: Mutex::new(server_id)})
        .manage(db)
}