use clap::{Parser, Subcommand};
use cln_init::manage::{
    node_manager_client::NodeManagerClient, CreateWalletRequest, CreateWalletResult,
    GenSeedLength, GenSeedRequest,
};

#[derive(Subcommand, Debug)]
enum SubCommand {
    GenSeed {
        len: i8,
    },
    CreateWallet {
        #[clap(long)]
        seed: String,
        #[clap(long)]
        passphrase: Option<String>,
    },
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
    let mut client = NodeManagerClient::connect("http://127.0.0.1:8080").await?;
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
        SubCommand::CreateWallet { seed, passphrase } => {
            let seed = seed.split(' ').map(|s| s.to_string()).collect();
            let request = tonic::Request::new(CreateWalletRequest {
                bip39: seed,
                passphrase: passphrase.unwrap_or("".to_string()),
            });
            let response = client.create_wallet(request).await?;
            match CreateWalletResult::from_i32(response.into_inner().result)
                .expect("Invalid response from daemon")
            {
                CreateWalletResult::CreateWalletSuccess => {
                    eprintln!("The wallet was created successfully")
                }
                CreateWalletResult::CreateWalletErrorAlreadyExists => {
                    eprintln!("A wallet already exists")
                }
                CreateWalletResult::CreateWalletErrorPermissionDenied => {
                    eprintln!("Error while saving wallet: permission denied")
                }
                CreateWalletResult::CreateWalletErrorUnknown => {
                    eprintln!("An unknown error occurred while saving the wallet")
                }
                CreateWalletResult::CreateWalletErrorInvalidMnemonic => {
                    eprintln!("Invalid mnemonic seed!")
                }
            }
            Ok(())
        }
        SubCommand::DeleteWallet {} => todo!(),
    }
}
