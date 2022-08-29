use std::io::Read;

use cln_init::manage::{
    wallet_manager_server::{WalletManager, WalletManagerServer},
    CreateWalletRequest, CreateWalletResponse, CreateWalletResult, DeleteWalletRequest,
    DeleteWalletResponse, DeleteWalletResult, GenSeedRequest, GenSeedResponse, GenSeedLength,
};
use rand::SeedableRng;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Default)]
pub struct WalletManagerService {}

#[tonic::async_trait]
impl WalletManager for WalletManagerService {
    async fn create_wallet(
        &self,
        request: Request<CreateWalletRequest>,
    ) -> Result<Response<CreateWalletResponse>, Status> {
        let _r = request.into_inner();
        println!("Received request to create wallet");
        Ok(Response::new(CreateWalletResponse {
            result: CreateWalletResult::CreateWalletSuccess as i32,
        }))
    }
    async fn delete_wallet(
        &self,
        request: Request<DeleteWalletRequest>,
    ) -> Result<Response<DeleteWalletResponse>, Status> {
        let _r = request.into_inner();
        println!("Received request to delete wallet");
        Ok(Response::new(DeleteWalletResponse {
            result: DeleteWalletResult::DeleteWalletSuccess as i32,
        }))
    }
    async fn gen_seed(
        &self,
        request: Request<GenSeedRequest>,
    ) -> Result<Response<GenSeedResponse>, Status> {
        let r = request.into_inner();
        println!("Received request to generate wallet");
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting server at port 8080");
    let address = "[::1]:8080".parse().unwrap();
    let voting_service = WalletManagerService::default();

    Server::builder()
        .add_service(WalletManagerServer::new(voting_service))
        .serve(address)
        .await?;
    Ok(())
}
