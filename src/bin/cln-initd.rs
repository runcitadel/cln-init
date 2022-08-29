use std::{io::{Read, Write}, path::Path, env};

use cln_init::manage::{
    node_manager_server::{NodeManager, NodeManagerServer},
    CreateWalletRequest, CreateWalletResponse, CreateWalletResult, DeleteWalletRequest,
    DeleteWalletResponse, DeleteWalletResult, GenSeedRequest, GenSeedResponse, GenSeedLength, StartDaemonRequest, StartDaemonResponse, StartDaemonResult,
};
use rand::SeedableRng;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use rand_chacha::ChaCha20Rng;
use lazy_static::lazy_static;
use sysinfo::{System, SystemExt};

lazy_static! {
    static ref CONFIG_DIR: String = env::args().nth(1).expect("First argument missing!");
}

#[derive(Debug, Default)]
pub struct NodeManagerService {}

#[tonic::async_trait]
impl NodeManager for NodeManagerService {
    async fn create_wallet(
        &self,
        request: Request<CreateWalletRequest>,
    ) -> Result<Response<CreateWalletResponse>, Status> {
        let r = request.into_inner();
        let mnemonic = bip39::Mnemonic::parse(r.bip39.join(" "));
        if mnemonic.is_err() {
            return Ok(Response::new(CreateWalletResponse {
                result: CreateWalletResult::CreateWalletErrorInvalidMnemonic as i32,
            }));
        }
        let mnemonic = mnemonic.unwrap();
        let seed = mnemonic.to_seed(r.passphrase);
        let mut hsm_secret: [u8; 32] = [0; 32];
        assert_eq!(seed.take(32).read(&mut hsm_secret).expect("Failed to write"), 32);
        let config_dir = Path::new(CONFIG_DIR.as_str());
        let hsm_file = config_dir.join("bitcoin").join("hsm_secret");
        if hsm_file.exists() {
            return Ok(Response::new(CreateWalletResponse {
                result: CreateWalletResult::CreateWalletErrorAlreadyExists as i32,
            }));
        }
        let file = std::fs::File::create(hsm_file);
        if file.is_err() {
            eprintln!("{}", file.err().unwrap());
            return Ok(Response::new(CreateWalletResponse {
                result: CreateWalletResult::CreateWalletErrorUnknown as i32,
            }));
        }
        let mut file = file.unwrap();
        file.write_all(&hsm_secret).unwrap();
        Ok(Response::new(CreateWalletResponse {
            result: CreateWalletResult::CreateWalletSuccess as i32,
        }))
    }
    async fn delete_wallet(
        &self,
        request: Request<DeleteWalletRequest>,
    ) -> Result<Response<DeleteWalletResponse>, Status> {
        let _r = request.into_inner();
        Ok(Response::new(DeleteWalletResponse {
            result: DeleteWalletResult::DeleteWalletSuccess as i32,
        }))
    }
    async fn gen_seed(
        &self,
        request: Request<GenSeedRequest>,
    ) -> Result<Response<GenSeedResponse>, Status> {
        let r = request.into_inner();
        let entropy = ChaCha20Rng::from_entropy();
        let len = match GenSeedLength::from_i32(r.length).expect("Bad data") {
            GenSeedLength::GenSeed12Words => 128,
            GenSeedLength::GenSeed15Words => 160,
            GenSeedLength::GenSeed18Words => 192,
            GenSeedLength::GenSeed21Words => 224,
            GenSeedLength::GenSeed24Words => 256,
        } / 8;
        let seed = entropy.get_seed();
        let seed = seed.take(len);
        let mnemonic = bip39::Mnemonic::from_entropy(seed.into_inner()).expect("Seed generation failed!");
        let mnemonic: Vec<String> = mnemonic.to_string().split(' ').map(|s| s.to_string()).collect();
        Ok(Response::new(GenSeedResponse {
            bip39: mnemonic,
        }))
    }
    async fn start_daemon(
        &self,
        request: Request<StartDaemonRequest>,
    ) -> Result<Response<StartDaemonResponse>, Status> {
        let r = request.into_inner();
        // First, check if lightningd is already running
        let mut sys = System::new();
        sys.refresh_processes();
        let processes = sys.processes_by_name("lightningd");
        if processes.count() > 0 {
            return Ok(Response::new(StartDaemonResponse {
                result: StartDaemonResult::StartDaemonErrorAlreadyRunning as i32,
            }));
        }
        // Otherwise, just launch lightningd in a new process
        let mut cmd = std::process::Command::new("lightningd");
        cmd.args(r.args);
        cmd.current_dir(CONFIG_DIR.as_str());
        let child = cmd.spawn();
        if child.is_err() {
            return Ok(Response::new(StartDaemonResponse {
                result: StartDaemonResult::StartDaemonErrorUnknown as i32,
            }));
        }
        Ok(Response::new(StartDaemonResponse {
            result: StartDaemonResult::StartDaemonSuccess as i32,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !System::IS_SUPPORTED {
        panic!("This OS isn't supported (yet?).");
    }
    let args = env::args().collect::<Vec<_>>();
    // Ensure there is exactly one argument, otherwise fail
    if args.len() != 2 {
        println!("Usage: {} <cln-config-dir>", args[0]);
        return Ok(());
    }
    let config_dir = Path::new(CONFIG_DIR.as_str());
    assert!(config_dir.is_dir(), "Config dir is not a directory");
    println!("Listening on port 8080");
    let address = "[::1]:8080".parse().unwrap();
    let node_manager_service = NodeManagerService::default();

    Server::builder()
        .add_service(NodeManagerServer::new(node_manager_service))
        .serve(address)
        .await?;
    Ok(())
}
