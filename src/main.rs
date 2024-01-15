use clap::{arg, command};
use image::{imageops::colorops::invert, io::Reader};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

const MATRICE_WIDTH: usize = 100;
const MATRICE_HEIGHT: usize = 50;

fn main() -> std::io::Result<()> {
    let matches = command!()
        .arg(arg!([img] "Image to generate cnc code for"))
        .get_matches();
    let mut img_path = PathBuf::new();
    img_path.push::<PathBuf>(
        matches
            .get_one::<String>("img")
            .expect("No image was provided!")
            .into(),
    );
    let img = Reader::open(img_path.clone())
        .expect(&format!(
            "Failed to load image: {}\n",
            img_path.clone().to_str().unwrap()
        ))
        .decode()
        .expect(&format!(
            "Failed to decode image: {}\n",
            img_path.clone().to_str().unwrap()
        ));
    let img = &mut img.to_luma8();
    invert(img);
    img.save(format!("luma_{}", img_path.clone().to_str().unwrap()))
        .unwrap();

    for x in 0..img.dimensions().0 {
        for y in 0..img.dimensions().1 {
            println!("{:?}", img.get_pixel(x, y));
        }
    }

    let mut matrix: [[u32; MATRICE_HEIGHT]; MATRICE_WIDTH] = [[0; MATRICE_HEIGHT]; MATRICE_WIDTH];
    (0..MATRICE_WIDTH).for_each(|x| {
        (0..MATRICE_HEIGHT).for_each(|y| {
            matrix[x][y] = {
                let mut out: u32 = 0;
                (0..(img.dimensions().0 / (MATRICE_WIDTH as u32))).for_each(|i| {
                    (0..(img.dimensions().1 / MATRICE_HEIGHT as u32)).for_each(|j| {
                        //println!("{}", img.get_pixel(x as u32 + i, y as u32 + j).0[0] as u32);
                        out = out + img.get_pixel(x as u32 + i, y as u32 + j).0[0] as u32;
                    });
                });
                println!("x: {} y: {} val: {}", x, y, out);
                out
            }
        });
    });
    let mut cnc_file = File::create(format!(
        "{}.cnc",
        img_path.to_str().unwrap().split(".").nth(0).unwrap()
    ))?;
    cnc_file.write_all(b"G90\nG0 Z0.5\nM3 S255\nG4 S5\n")?;
    (0..MATRICE_WIDTH).for_each(|x| {
        (0..MATRICE_HEIGHT).for_each(|y| {
            if matrix[x][y] != 0 {
                cnc_file
                    .write_all(
                        format!("G0 X{} Y{}\nG1 Z{:.2} F100\nG0 Z0.5\n", x, y, {
                            let out = matrix[x][y] as f32;
                            println!("{}", -(out / 1020.0));
                            -(out / 1020.0) * 3.0
                        })
                        .as_bytes(),
                    )
                    .unwrap();
            }
        });
    });
    cnc_file.write_all(b"M30")?;

    Ok(())
}
