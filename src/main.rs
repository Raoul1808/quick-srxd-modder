use std::{
    cmp::min,
    ffi::OsStr,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use futures_util::StreamExt;
use indicatif::ProgressBar;
use zip_extensions::zip_extract;

const BEPINEX_URI: &str =
    "https://github.com/BepInEx/BepInEx/releases/download/v5.4.23.2/BepInEx_win_x64_5.4.23.2.zip";
const SPINCORE_URI: &str =
    "https://github.com/Raoul1808/SpinCore/releases/download/v1.1.0/SpinCore.zip";
const DTS_URI: &str =
    "https://github.com/Raoul1808/DynamicTrackSpeed/releases/download/v1.5.0/DynamicTrackSpeed.dll";
const SC2_URI: &str =
    "https://github.com/Raoul1808/SpeenChroma2/releases/download/v2.1.0/SpeenChroma2.dll";

async fn download_file(uri: &str, filename: &str) -> PathBuf {
    let res = reqwest::get(uri)
        .await
        .unwrap_or_else(|_| panic!("failed to reach to address: {}", uri));
    let size = res
        .content_length()
        .expect("failed to get content length of remote file");
    let pb = ProgressBar::new(size);
    pb.set_message(format!("Downloading {}", filename));

    let mut file = File::create_new(filename).expect("cannot download file");
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("error while downloading file");
        file.write_all(&chunk).expect("error while writing to file");
        let new = min(downloaded + (chunk.len() as u64), size);
        downloaded = new;
        pb.set_position(new);
    }
    pb.finish_with_message(format!("Downloaded {}", filename));
    PathBuf::from_str(filename).expect("failed to allocate pathbuf")
}

async fn setup_bepinex(game_dir: &Path) {
    let bepinex = download_file(BEPINEX_URI, "BepInEx.zip").await;
    println!("Extracting...");
    zip_extract(&bepinex, &game_dir.to_path_buf()).expect("failed to extract BepInEx.zip");
    println!("Deleting temp file...");
    fs::remove_file(bepinex).expect("failed to delete BepInEx.zip");
    println!("Creating plugins directory...");
    fs::create_dir(game_dir.join("BepInEx").join("plugins"))
        .expect("failed to create plugins directory");
    println!("BepInEx installed!");
}

async fn setup_spincore(game_dir: &Path) {
    let spincore = download_file(SPINCORE_URI, "SpinCore.zip").await;
    println!("Extracting...");
    let plugins_dir = game_dir.join("BepInEx").join("plugins");
    let spincore_dll = plugins_dir.join("SpinCore.dll");
    if spincore_dll.exists() {
        fs::remove_file(spincore_dll).expect("failed to delete existing SpinCore.dll file");
    }
    let json_dll = plugins_dir.join("Newtonsoft.Json.dll");
    if json_dll.exists() {
        fs::remove_file(json_dll).expect("failed to delete existing Newtonsoft.Json.dll file");
    }
    zip_extract(&spincore, &game_dir.join("BepInEx").join("plugins"))
        .expect("failed to extract SpinCore.zip");
    println!("Deleting temp file...");
    fs::remove_file(spincore).expect("failed to delete SpinCore.zip");
    println!("SpinCore installed!");
}

async fn setup_dts(game_dir: &Path) {
    let dts = download_file(DTS_URI, "DynamicTrackSpeed.dll").await;
    println!("Installing mod...");
    fs::copy(
        &dts,
        game_dir
            .join("BepInEx")
            .join("plugins")
            .join("DynamicTrackSpeed.dll"),
    )
    .expect("failed to copy file");
    fs::remove_file(dts).expect("failed to delete original file");
    println!("DynamicTrackSpeed installed!");
}

async fn setup_chroma(game_dir: &Path) {
    let chroma = download_file(SC2_URI, "SpeenChroma2.dll").await;
    println!("Installing mod...");
    fs::copy(
        &chroma,
        game_dir
            .join("BepInEx")
            .join("plugins")
            .join("SpeenChroma2.dll"),
    )
    .expect("failed to copy file");
    fs::remove_file(chroma).expect("failed to delete original file");
    println!("SpeenChroma2 installed!");
}

fn setup_dlls(game_dir: &Path) {
    let unity_dll = game_dir.join("UnityPlayer.dll");
    let mono_dll = game_dir.join("UnityPlayer_Mono.dll");
    let il2cpp_dll = game_dir.join("UnityPlayer_IL2CPP.dll");
    if il2cpp_dll.exists() {
        fs::remove_file(&il2cpp_dll).expect("failed to delete UnityPlayer_IL2CPP.dll");
    }
    fs::rename(&unity_dll, &il2cpp_dll)
        .expect("failed to rename UnityPlayer.dll to UnityPlayer_IL2CPP.dll");
    fs::rename(&mono_dll, &unity_dll)
        .expect("failed to rename UnityPlayer_Mono.dll to UnityPlayer.dll");
    println!("UnityPlayer.dll files swapped!");
}

#[tokio::main]
async fn main() {
    println!("Please select SpinRhythm.exe");
    let game = rfd::FileDialog::new()
        .add_filter("executable files (*.exe)", &["exe"])
        .pick_file()
        .expect("Please select the game");
    if game.file_name() != Some(OsStr::new("SpinRhythm.exe")) {
        panic!("Please select SpinRhythm.exe");
    }
    let game_dir = game.parent().expect("failed to get parent directory");
    if !game_dir.join("BepInEx").exists() {
        setup_bepinex(game_dir).await;
    }
    let plugins_path = game_dir.join("BepInEx").join("plugins");
    //if !plugins_path.join("SpinCore.dll").exists()
    //    || !plugins_path.join("Newtonsoft.Json.dll").exists()
    {
        setup_spincore(game_dir).await;
    }
    if !plugins_path.join("DynamicTrackSpeed.dll").exists() {
        setup_dts(game_dir).await;
    }
    if !plugins_path.join("SpeenChroma2.dll").exists() {
        setup_chroma(game_dir).await;
    }
    if game_dir.join("UnityPlayer_Mono.dll").exists() {
        setup_dlls(game_dir);
    }
    println!("All done!");
}
