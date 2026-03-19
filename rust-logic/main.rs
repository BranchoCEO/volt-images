use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use image::GenericImageView;
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: volt <path_to_image>");
        return;
    }

    let img_path_str = &args[1];
    let img_path = Path::new(img_path_str);

    let img = match image::open(img_path) {
        Ok(i) => i,
        Err(e) => {
            println!("Error: Could not open image: {}", e);
            return;
        }
    };

    let (width, height) = img.dimensions();

    // Decode to raw RGBA8 bytes once — avoids per-pixel trait dispatch overhead
    let rgba = img.to_rgba8();
    let raw = rgba.as_raw();

    let file_stem = img_path.file_stem().unwrap().to_str().unwrap();
    let txt_name = format!("{}.txt", file_stem);

    println!("Processing {}...", img_path_str);

    // Process each row in parallel, each producing its own byte buffer
    let rows: Vec<Vec<u8>> = (0..height)
        .into_par_iter()
        .map(|y| {
            let mut row_buf = Vec::with_capacity(width as usize * 35);
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                let r = raw[idx];
                let g = raw[idx + 1];
                let b = raw[idx + 2];
                let a = raw[idx + 3];
                if a > 0 {
                    write!(
                        row_buf,
                        "PX: x{:03} y{:03} | COLOR: #{:02X}{:02X}{:02X}\n",
                        x, y, r, g, b
                    )
                    .unwrap();
                }
            }
            row_buf
        })
        .collect();

    // Single file open + bulk write — one I/O operation per row buffer
    let txt_file = File::create(&txt_name).expect("Failed to create text file");
    let mut writer = BufWriter::with_capacity(1024 * 1024, txt_file);
    for row in rows {
        writer.write_all(&row).unwrap();
    }

    println!("Finished! Created: {}", txt_name);
}
