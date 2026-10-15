//! Sonde du suivi des mains : FPS, taux de détection, geste, et dump PPM annoté.
//! `cargo run --release -p nexora-vision --features camera,onnx --example hand_probe -- [index] [frames] [--serve [port]]`
//!
//! Avec `--serve`, un flux MJPEG annoté (miroir) est visible sur `http://127.0.0.1:<port>`
//! (8080 par défaut) : 21 points, squelette et ROI rotatée.

use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use nexora_vision::camera::{Camera, RgbFrame};
use nexora_vision::hand::{default_models_dir, HandResult, OnnxHandTracker};
use nexora_vision::{classify, idx};

/// Arêtes du squelette de la main (convention MediaPipe).
const BONES: [(usize, usize); 21] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (0, 5),
    (5, 6),
    (6, 7),
    (7, 8),
    (5, 9),
    (9, 10),
    (10, 11),
    (11, 12),
    (9, 13),
    (13, 14),
    (14, 15),
    (15, 16),
    (13, 17),
    (17, 18),
    (18, 19),
    (19, 20),
    (0, 17),
];

fn dot(f: &mut RgbFrame, x: f32, y: f32, color: [u8; 3]) {
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            put(f, x as i32 + dx, y as i32 + dy, color);
        }
    }
}

fn put(f: &mut RgbFrame, x: i32, y: i32, color: [u8; 3]) {
    if x >= 0 && y >= 0 && (x as u32) < f.width && (y as u32) < f.height {
        let i = (y as usize * f.width as usize + x as usize) * 3;
        f.data[i..i + 3].copy_from_slice(&color);
    }
}

fn line(f: &mut RgbFrame, a: [f32; 2], b: [f32; 2], color: [u8; 3]) {
    let n = ((b[0] - a[0]).abs().max((b[1] - a[1]).abs()))
        .ceil()
        .max(1.0) as i32;
    for i in 0..=n {
        let t = i as f32 / n as f32;
        put(
            f,
            (a[0] + (b[0] - a[0]) * t) as i32,
            (a[1] + (b[1] - a[1]) * t) as i32,
            color,
        );
    }
}

fn mirror(f: &mut RgbFrame) {
    let row = f.width as usize * 3;
    for line in f.data.chunks_mut(row) {
        for x in 0..f.width as usize / 2 {
            for c in 0..3 {
                line.swap(x * 3 + c, (f.width as usize - 1 - x) * 3 + c);
            }
        }
    }
}

/// Dessine la main sur une frame déjà en miroir.
fn annotate(f: &mut RgbFrame, h: &HandResult) {
    let (w, hh) = (f.width as f32, f.height as f32);
    let pt = |p: [f32; 3]| [w - p[0] * w, p[1] * hh];
    for (a, b) in BONES {
        line(f, pt(h.landmarks[a]), pt(h.landmarks[b]), [255, 255, 0]);
    }
    for p in &h.landmarks {
        let [x, y] = pt(*p);
        dot(f, x, y, [255, 0, 0]);
    }
    let corners = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].map(|(u, v)| {
        let [x, y] = h.roi.to_image(u, v);
        [w - x, y]
    });
    for i in 0..4 {
        line(f, corners[i], corners[(i + 1) % 4], [0, 255, 0]);
    }
}

fn jpeg(f: &RgbFrame) -> Vec<u8> {
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 75)
        .encode(&f.data, f.width, f.height, image::ExtendedColorType::Rgb8)
        .expect("encodage JPEG");
    out
}

fn serve(port: u16) -> Arc<Mutex<Vec<u8>>> {
    let latest = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("port du flux");
    println!("flux : http://127.0.0.1:{port}");
    let shared = latest.clone();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let shared = shared.clone();
            thread::spawn(move || {
                let mut s = stream;
                let head = "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=f\r\n\r\n";
                if s.write_all(head.as_bytes()).is_err() {
                    return;
                }
                loop {
                    let img = shared.lock().unwrap().clone();
                    if !img.is_empty() {
                        let part = format!(
                            "--f\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                            img.len()
                        );
                        if s.write_all(part.as_bytes())
                            .and_then(|_| s.write_all(&img))
                            .and_then(|_| s.write_all(b"\r\n"))
                            .is_err()
                        {
                            return;
                        }
                    }
                    thread::sleep(Duration::from_millis(40));
                }
            });
        }
    });
    latest
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let pos: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let index = pos.first().and_then(|a| a.parse().ok());
    let frames: u32 = pos.get(1).and_then(|a| a.parse().ok()).unwrap_or(300);
    let port: u16 = pos.get(2).and_then(|a| a.parse().ok()).unwrap_or(8080);
    let stream = args.iter().any(|a| a == "--serve").then(|| serve(port));

    let mut cam = Camera::open(index, 640, 480, 30).expect("ouverture caméra");
    let mut tracker = OnnxHandTracker::new(&default_models_dir()).expect("modèles ONNX");
    println!("caméra [{}] {}", cam.info().index, cam.info().name);

    let (mut hits, mut last_hit) = (0u32, None);
    let start = Instant::now();
    for n in 0..frames {
        let mut frame = cam.frame().expect("frame");
        let hand = tracker.detect(&frame).expect("inférence");
        mirror(&mut frame);
        if let Some(h) = &hand {
            hits += 1;
            if n % 15 == 0 {
                let tip = h.landmarks[idx::INDEX_TIP];
                println!(
                    "frame {n:4} : main score {:.2} droite {:.2} geste {:?} index ({:.2}, {:.2})",
                    h.score,
                    h.right_prob,
                    classify(&h.landmarks),
                    tip[0],
                    tip[1]
                );
            }
            annotate(&mut frame, h);
            last_hit = Some(frame.clone());
        }
        if let Some(s) = &stream {
            *s.lock().unwrap() = jpeg(&frame);
        }
    }
    let secs = start.elapsed().as_secs_f32();
    println!(
        "{frames} frames en {secs:.2} s -> {:.1} fps, main détectée sur {:.0} % des frames",
        frames as f32 / secs,
        100.0 * hits as f32 / frames as f32
    );

    if let Some(f) = last_hit {
        let path = "target/hand_probe.ppm";
        let mut out = std::fs::File::create(path).expect("fichier PPM");
        write!(out, "P6\n{} {}\n255\n", f.width, f.height).unwrap();
        out.write_all(&f.data).unwrap();
        println!("dernière frame avec main (miroir) -> {path}");
    }
}
