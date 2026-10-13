//! Sonde webcam : liste les caméras, mesure les FPS et écrit la dernière frame en PPM.
//! `cargo run -p nexora-vision --features camera --example camera_probe -- [index] [frames]`

use std::io::Write;
use std::time::Instant;

use nexora_vision::camera::{list_cameras, Camera};

fn main() {
    let mut args = std::env::args().skip(1);
    let index = args.next().and_then(|a| a.parse().ok());
    let frames: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(150);

    println!("— Caméras détectées —");
    match list_cameras() {
        Ok(cams) => cams
            .iter()
            .for_each(|c| println!("  [{}] {}", c.index, c.name)),
        Err(e) => println!("  erreur : {e}"),
    }

    let mut cam = Camera::open(index, 640, 480, 30).expect("ouverture caméra");
    println!(
        "— Ouverte : [{}] {} ({})",
        cam.info().index,
        cam.info().name,
        cam.format()
    );

    let start = Instant::now();
    let mut last = None;
    for _ in 0..frames {
        last = Some(cam.frame().expect("frame"));
    }
    let secs = start.elapsed().as_secs_f32();
    println!(
        "{frames} frames en {secs:.2} s -> {:.1} fps ({:.1} ms/frame, capture + décodage RGB)",
        frames as f32 / secs,
        1000.0 * secs / frames as f32
    );

    if let Some(f) = last {
        let path = "target/camera_probe.ppm";
        let mut out = std::fs::File::create(path).expect("fichier PPM");
        write!(out, "P6\n{} {}\n255\n", f.width, f.height).unwrap();
        out.write_all(&f.data).unwrap();
        println!("dernière frame ({}x{}) -> {path}", f.width, f.height);
    }
}
