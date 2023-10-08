extern crate core;
extern crate hex;

mod mpc;
use mpc::operations;

mod communication;
use communication::create_communication_channel;
mod pub_endpoints;
mod rocket_instances;
mod mpc_config;
use mpc_config::MPCconfig;

use std::{
    sync::{Arc},
};

use anyhow::Result;

use rocket::{
    data::{ByteUnit, Limits},
};

use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // id that will be used to filter out messages
    let server_id = args.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    let port = args.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(8000);
    let port_mutual_auth = args.get(3).and_then(|s| s.parse::<u16>().ok()).unwrap_or(3000);
    let threshold = args.get(4).and_then(|s| s.parse::<u16>().ok()).unwrap_or(1);
    let number_of_parties = args.get(5).and_then(|s| s.parse::<u16>().ok()).unwrap_or(3);

    let config = MPCconfig::new(server_id, threshold, number_of_parties);

    // TODO: might be good idea to adjust for development and production (https://rocket.rs/v0.4/guide/configuration/)
    // Create a figment with the desired configuration
    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("workers", 4))
        .merge(("log_level", "normal"))
        .merge(("limits", Limits::new().limit("json", ByteUnit::from(1048576 * 1024))));


    let shared_db = rocket_instances::SharedDb(
        Arc::new(
            create_communication_channel::Db::empty(config.threshold())
        )
    );

    // Create two Rocket instances with different ports and TLS settings
    let rocket_instance_protected =
        rocket_instances::rocket_with_client_auth(figment.clone(),
                                                  config.clone(),
                                                  shared_db.clone(),
                                                  port_mutual_auth);
    let rocket_instance_public =
        rocket_instances::rocket_without_client_auth(figment.clone(),
                                                     config.clone(),
                                                     shared_db.clone(),
                                                     port);

    let signer = Arc::new(RwLock::new(operations::signing::Signer::new(server_id)));

    let rocket_instance_protected = rocket_instance_protected.manage(signer.clone());
    let rocket_instance_public = rocket_instance_public.manage(signer.clone());

    // Run the Rocket instances concurrently
    let server_future_protected = tokio::spawn(async { rocket_instance_protected.launch().await });
    let server_future_public = tokio::spawn(async { rocket_instance_public.launch().await });

    let (protected_result, public_result) = tokio::join!(server_future_protected, server_future_public);

    // Check the results
    println!("Protected Rocket server result: {:?}", protected_result);
    println!("Public Rocket server result: {:?}", public_result);

    Ok(())
}
