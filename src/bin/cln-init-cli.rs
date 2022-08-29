use clap::{Parser, Subcommand};
use cln_init::manage::{
    wallet_manager_client::WalletManagerClient, CreateWalletRequest, CreateWalletResult,
    GenSeedLength, GenSeedRequest,
};

#[derive(Subcommand, Debug)]
enum SubCommand {
    GenSeed { len: i8 },
    CreateWallet { seed: String },
    DeleteWallet {},
}
#[derive(Parser)]
struct Cli {
    /// The subcommand to run
    #[clap(subcommand)]
    command: SubCommand,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Cli = Cli::parse();
    let mut client = WalletManagerClient::connect("http://[::1]:8080").await?;
    match args.command {
        SubCommand::GenSeed { len } => {
            let length = match len {
                12 => GenSeedLength::GenSeed12Words,
                15 => GenSeedLength::GenSeed15Words,
                18 => GenSeedLength::GenSeed18Words,
                21 => GenSeedLength::GenSeed21Words,
                24 => GenSeedLength::GenSeed24Words,
                _ => panic!("Invalid seed length"),
            };
            let request = tonic::Request::new(GenSeedRequest {
                length: length as i32,
            });
            let response = client.gen_seed(request).await?;
            println!(
                "This is your seed: {}",
                response.into_inner().bip39.join(" ")
            );
            Ok(())
        }
        SubCommand::CreateWallet { seed } => {
            let seed = seed.split(' ').map(|s| s.to_string()).collect();
            let request = tonic::Request::new(CreateWalletRequest { bip39: seed });
            let response = client.create_wallet(request).await?;
            match CreateWalletResult::from_i32(response.into_inner().result)
                .expect("Invalid response from daemon")
            {
                CreateWalletResult::CreateWalletSuccess => {
                    println!("The wallet was created successfully")
                }
                CreateWalletResult::CreateWalletErrorAlreadyExists => {
                    println!("A wallet already exists")
                }
                CreateWalletResult::CreateWalletErrorPermissionDenied => println!("Error while saving wallet: permission denied"),
                CreateWalletResult::CreateWalletErrorUnknown => println!("An unknown error occurred while saving the wallet"),
            }
            Ok(())
        }
        SubCommand::DeleteWallet {} => todo!(),
    }
}
