use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Known WUDAPT download URLs (as of 2024)
const WUDAPT_URLS: &[(&str, &str)] = &[
    ("lcz-generator-v3", "https://lcz-generator.rub.de/cogs/lcz_filter_v3_cog.tif"),
    ("zenodo-v3", "https://zenodo.org/records/6364594/files/lcz_filter_v3.tif"),
    ("lcz-generator-v2", "https://lcz-generator.rub.de/cogs/lcz_filter_v2_cog.tif"),
];

/// Default locations to place the downloaded file
fn get_default_locations() -> Vec<PathBuf> {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    vec![
        current_dir.join("wudapt_lcz_global.tif"),
        current_dir.join("data").join("wudapt_lcz_global.tif"),
        PathBuf::from("/tmp/wudapt_lcz_global.tif"),
        dirs::home_dir().unwrap_or_default().join(".cache").join("urban_classifier").join("wudapt_lcz_global.tif"),
    ]
}

fn download_with_progress(url: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Downloading Global LCZ Map from: {}", url);
    println!("📁 Saving to: {}", output_path.display());
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(300)) // 5 minute timeout
        .build()?;

    // Get the file size for progress bar
    let response = client.head(url).send()?;
    let total_size = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| ct.parse::<u64>().ok())
        .unwrap_or(0);

    // Start the download
    let mut response = client.get(url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download: HTTP {}", response.status()).into());
    }

    // Setup progress bar
    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-"));
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] Downloading... {bytes} ({bytes_per_sec})")
            .unwrap());
        pb
    };

    // Write to file with progress updates
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    let mut downloaded = 0u64;
    let mut buffer = [0; 8192];

    loop {
        match response.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                writer.write_all(&buffer[..n])?;
                downloaded += n as u64;
                pb.set_position(downloaded);
            }
            Err(e) => return Err(e.into()),
        }
    }

    writer.flush()?;
    pb.finish_with_message("✅ Download complete!");

    // Verify the file was downloaded correctly
    let file_size = fs::metadata(output_path)?.len();
    if file_size == 0 {
        return Err("Downloaded file is empty".into());
    }

    println!("📊 File size: {:.2} MB", file_size as f64 / 1_048_576.0);
    
    Ok(())
}

fn verify_geotiff(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Verifying GeoTIFF file...");
    
    // Basic file existence and size check
    let metadata = fs::metadata(path)?;
    if metadata.len() == 0 {
        return Err("File is empty".into());
    }

    // Check file header for TIFF signature
    let mut file = File::open(path)?;
    let mut header = [0u8; 4];
    std::io::Read::read_exact(&mut file, &mut header)?;
    
    // TIFF files start with either "II*\0" (little-endian) or "MM\0*" (big-endian)
    if !(header == [0x49, 0x49, 0x2A, 0x00] || header == [0x4D, 0x4D, 0x00, 0x2A]) {
        return Err("File does not appear to be a valid TIFF file".into());
    }

    println!("✅ File appears to be a valid TIFF file");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("WUDAPT Global LCZ Map Downloader")
        .version("0.1.0")
        .author("Urban Classifier")
        .about("Downloads the Global Local Climate Zone (LCZ) map from WUDAPT for use with urban_classifier")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path (default: ./wudapt_lcz_global.tif)")
        )
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("URL")
                .help("Custom download URL (uses WUDAPT official URLs by default)")
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .action(clap::ArgAction::SetTrue)
                .help("Force download even if file already exists")
        )
        .arg(
            Arg::new("list-locations")
                .long("list-locations")
                .action(clap::ArgAction::SetTrue)
                .help("List default download locations")
        )
        .get_matches();

    if matches.get_flag("list-locations") {
        println!("Default download locations:");
        for (i, location) in get_default_locations().iter().enumerate() {
            println!("  {}: {}", i + 1, location.display());
        }
        return Ok(());
    }

    // Determine output path
    let output_path = if let Some(path) = matches.get_one::<String>("output") {
        PathBuf::from(path)
    } else {
        get_default_locations().into_iter().next().unwrap()
    };

    // Check if file already exists
    if output_path.exists() && !matches.get_flag("force") {
        println!("✅ File already exists: {}", output_path.display());
        println!("💡 Use --force to re-download, or specify a different output path with --output");
        
        // Verify existing file
        match verify_geotiff(&output_path) {
            Ok(()) => {
                println!("✅ Existing file appears to be valid");
                println!("🎯 Ready to use with urban_classifier!");
                return Ok(());
            }
            Err(e) => {
                println!("⚠️  Warning: Existing file may be corrupted: {}", e);
                println!("🔄 Consider re-downloading with --force");
                return Ok(());
            }
        }
    }

    // Determine download URL
    let urls = if let Some(custom_url) = matches.get_one::<String>("url") {
        vec![("custom", custom_url.as_str())]
    } else {
        WUDAPT_URLS.to_vec()
    };

    // Try downloading from each URL until one succeeds
    let mut last_error = None;
    for (source, url) in urls {
        println!("\n🚀 Attempting download from {} source...", source);
        
        match download_with_progress(url, &output_path) {
            Ok(()) => {
                // Verify the downloaded file
                match verify_geotiff(&output_path) {
                    Ok(()) => {
                        println!("\n🎉 SUCCESS! Global LCZ Map downloaded and verified!");
                        println!("📁 Location: {}", output_path.display());
                        println!();
                        println!("🔧 You can now use this file with urban_classifier:");
                        println!("   Rust: UrbanClassifier::new(\"{}\")", output_path.display());
                        println!("   Python: urban_classifier.PyUrbanClassifier(\"{}\")", output_path.display());
                        println!();
                        println!("🌍 Data Source: World Urban Database and Access Portal Tools (WUDAPT)");
                        println!("📖 Citation: Stewart, I.D. and Oke, T.R., 2012. Local climate zones");
                        println!("             for urban temperature studies. BAMS, 93(12), pp.1879-1900.");
                        return Ok(());
                    }
                    Err(e) => {
                        println!("⚠️  Downloaded file failed verification: {}", e);
                        let _ = fs::remove_file(&output_path); // Clean up bad file
                        last_error = Some(format!("Verification failed: {}", e).into());
                        continue;
                    }
                }
            }
            Err(e) => {
                println!("❌ Download failed from {}: {}", source, e);
                last_error = Some(e);
                continue;
            }
        }
    }

    // If we get here, all downloads failed
    if let Some(error) = last_error {
        eprintln!("\n💥 All download attempts failed. Last error: {}", error);
        eprintln!("🔧 Troubleshooting tips:");
        eprintln!("   1. Check your internet connection");
        eprintln!("   2. Try again later (servers may be temporarily unavailable)");
        eprintln!("   3. Download manually from: https://lcz-generator.rub.de/downloads");
        eprintln!("   4. Use --url to specify a custom download URL");
        return Err(error);
    }

    Ok(())
}

// Import Read trait for reqwest::Response
use std::io::Read;