use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use image::GenericImageView;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: volt <path_to_image>");
        return;
    }

    let img_path_str = &args[1];
    let img_path = Path::new(img_path_str);

    // 1. Open the image
    let img = match image::open(img_path) {
        Ok(i) => i,
        Err(e) => {
            println!("Error: Could not open image: {}", e);
            return;
        }
    };

    let (width, height) = img.dimensions();

    // 2. Create the Text File name based on the input image name
    // Example: "icon.png" -> "icon.txt"
    let file_stem = img_path.file_stem().unwrap().to_str().unwrap();
    let txt_name = format!("{}.txt", file_stem);
    
    let txt_file = File::create(&txt_name).expect("Failed to create text file");
    let mut writer = BufWriter::new(txt_file);

    println!("Reading {} ({}x{})...", img_path_str, width, height);

    // 3. Loop through pixels and write "Assembly-style" definitions
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let [r, g, b, a] = pixel.0;

            // We only record pixels that aren't transparent
            if a > 0 {
                // Formatting: X, Y coordinates and the HEX color code
                writeln!(
                    writer,
                    "PX: x{:03} y{:03} | COLOR: #{:02X}{:02X}{:02X}",
                    x, y, r, g, b
                ).unwrap();
            }
        }
    }

    println!("Finished! Data saved to: {}", txt_name);
}