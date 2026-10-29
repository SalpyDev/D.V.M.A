use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::container::load_dvma_png;
use crate::core::error::Result;
use crate::engine::{create_artifact, RecoveryEngine};

#[derive(Parser, Debug)]
#[command(
    name = "dvma",
    author = "DVMA Team",
    version = "1.0",
    about = "Digital Visual Matrix Authentication & Encryption Protocol v1.0",
    long_about = "DVMA turns a document into AEAD-encrypted data, encodes it as A-Z micro-notch dots, and splits the key into a user passkey (Share A) and PNG chunk (Share B)."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Encrypt a document and generate a DVMA PNG matrix image + user Share A passkey
    Create {
        /// Path to the input source document file
        #[arg(short, long)]
        input: PathBuf,

        /// Target output path for the generated DVMA PNG image
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Recover a document from a DVMA PNG image using Share A passkey
    Recover {
        /// Path to the input DVMA PNG image
        #[arg(short, long)]
        image: PathBuf,

        /// User-held Share A passkey (e.g., "DVMA-SHARE-A-...")
        #[arg(short = 's', long)]
        share_a: Option<String>,

        /// Target output path to save restored document
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Inspect a DVMA PNG image and extract metadata and Share B information
    Inspect {
        /// Path to the DVMA PNG image to inspect
        #[arg(short, long)]
        image: PathBuf,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { input, output } => {
            println!("🔒 Reading input file: {:?}", input);
            let input_bytes = fs::read(&input)?;

            println!("⚡ Encrypting document and generating DVMA matrix PNG...");
            let creation_res = create_artifact(&input_bytes, &output)?;

            println!("\n✅ DVMA PNG created successfully at: {:?}", output);
            println!("========================================================");
            println!("🔑 USER PASSKEY (SHARE A) - KEEP THIS SAFE:");
            println!("{}", creation_res.share_a_user_string);
            println!("========================================================");
            println!("⚠️ Loss of Share A makes document recovery IMPOSSIBLE from the image alone!");
        }

        Commands::Recover {
            image,
            share_a,
            output,
        } => {
            println!("📥 Ingesting DVMA PNG image: {:?}", image);
            let mut engine = RecoveryEngine::new();
            engine.ingest_image(&image)?;

            let share_a_input = match share_a {
                Some(s) => s,
                None => {
                    print!("🔑 Please enter Share A passkey: ");
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    input.trim().to_string()
                }
            };

            println!("⚡ Validating shares and reconstructing master key...");
            engine.supply_share_a(&share_a_input)?;

            println!("🔍 Decoding matrix and performing AEAD authenticated decryption...");
            let restored_doc_bytes = engine.execute_recovery()?;

            fs::write(&output, restored_doc_bytes.as_slice())?;
            println!(
                "\n🎉 Document recovered and saved successfully to: {:?}",
                output
            );
        }

        Commands::Inspect { image } => {
            println!("🔍 Inspecting DVMA PNG file: {:?}", image);
            let (gray_img, share_b) = load_dvma_png(&image)?;

            println!(
                "Dimensions: {}x{} pixels",
                gray_img.width(),
                gray_img.height()
            );
            println!("Share B Share Set ID: {:X?}", share_b.share_set_id);
            println!("Share B Coordinate x: {}", share_b.x_coord);
            println!("Share B Data Length: {} bytes", share_b.data.len());
            println!("✅ Valid DVMA PNG container with embedded Share B.");
        }
    }

    Ok(())
}
