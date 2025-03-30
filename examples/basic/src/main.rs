use asset_macros::assets;
use asset_traits::{Asset, AssetCollection};

// Generate asset enums for different directories
assets!(AudioAssets, "assets/audio");
assets!(UiAssets, "assets/ui", include: r"\.(png|jpg|svg)$");
assets!(ConfigAssets, "assets/config", include: r"\.json$", ignore: r"temp");

// Function that works with any asset type
fn process_asset<T: Asset>(asset: T) {
    let path = asset.path();
    let data = asset.bytes();
    println!("Processing asset: {} ({} bytes)", path, data.len());
}

// Function that takes a specific asset type
fn play_audio(audio: AudioAssets) {
    println!("Playing audio: {}", audio.path());
}

fn main() {
    // Use specific assets directly
    process_asset(UiAssets::logo_png);
    process_asset(AudioAssets::sound_ogg);

    // Find an asset by path
    if let Some(config) = ConfigAssets::find_by_path("settings.json") {
        process_asset(config);
    }

    // Iterate over all assets of a type
    for audio in AudioAssets::all() {
        play_audio(*audio);
    }

    // Print information about all UI assets
    println!("UI Assets:");
    for asset in UiAssets::all() {
        println!("  - {}: {} bytes", asset.path(), asset.bytes().len());
    }
}
