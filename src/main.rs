extern crate ctrlc;
extern crate image_convert;
extern crate termsize;
use image_convert::{to_pgm, ImageResource, PGMConfig};
use std::fs::File;
use std::io::Read;
use std::path::Path;

struct TerminalSize {
    cols: u32,
    rows: u32,
}

struct Image {
    size_x: u32,
    size_y: u32,
    pixel_array: Vec<Vec<f32>>,
}

fn main() {
    ctrl_c_init();
    let cli_args: Vec<String> = std::env::args().collect();
    if cli_args.len() == 1 {
        if std::env::consts::OS == "windows"{
            println!("ERROR: invalid usage\nCorrect usage: image_display <path to the pgm>");
        }else{
            println!("ERROR: invalid usage\nCorrect usage: ./image_display <path to the pgm>");
        }
        std::process::exit(1);
    }
    let term_size: TerminalSize = get_terminal_size();
    let pgm_binary_name = convert_to_pgm(cli_args[1].to_string());
    let image = ascii_pgm_string_to_image(parse_binary_pgm(pgm_binary_name.clone()));
    std::fs::remove_file(&pgm_binary_name).unwrap();

    reset_terminal();
    let mut pixel_array_term =
        vec![vec![0 as f32; term_size.rows as usize]; term_size.cols as usize];
    for j in 0..term_size.rows {
        for i in 0..term_size.cols {
            pixel_array_term[i as usize][j as usize] = image.pixel_array
                [((i as f32 / term_size.cols as f32) * image.size_x as f32) as usize]
                [((j as f32 / term_size.rows as f32) * image.size_y as f32) as usize];
            draw_grayscale(pixel_array_term[i as usize][j as usize]);
        }
        print!("\n");
    }
    draw_grayscale(0f32);
}

fn reset_terminal() {
    print!("\x1B[2J\x1B[1;1H");
}

fn draw_grayscale(val: f32) {
    print_ansi_string(val, " ".to_string());
}

fn print_ansi_string(val: f32, text: String) {
    let mut temp_val = val;
    temp_val = 232.0 + temp_val * 23.0;
    print!("\x1b[48;5;{}m{}", temp_val as u16, text);
}

fn get_terminal_size() -> TerminalSize {
    let mut return_struct: TerminalSize = TerminalSize { rows: 0, cols: 0 };
    termsize::get().map(|size| {
        return_struct.rows = size.rows as u32;
        return_struct.cols = size.cols as u32;
    });
    return_struct
}


fn ctrl_c_init() {
    ctrlc::set_handler(move || {
        println!("Keyboard Interrupt");
        std::process::exit(1);
    })
    .expect("Error setting Ctrl-C handler");
}

fn read_a_file(file_path: String) -> Vec<u8> {
    let mut f = File::open(&file_path).expect("no file found");
    let metadata = std::fs::metadata(&file_path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    buffer
}

fn ascii_pgm_string_to_image(file_string: String) -> Image {
    let mut return_image = Image {
        size_x: 0,
        size_y: 0,
        pixel_array: vec![vec![]],
    };
    let mut useful_line_number: u32 = 1;
    let mut value_array: Vec<f32> = vec![];
    let mut white_value: f32 = 1.0;
    for line_text in file_string.lines() {
        if !line_text.contains("#") {
            if useful_line_number == 2 {
                let contains_image_size: String = line_text.to_string();
                for string_type_image_size in contains_image_size.split_whitespace() {
                    if useful_line_number != 0 {
                        return_image.size_x = string_type_image_size.parse::<u32>().unwrap();
                        useful_line_number = 0;
                    } else {
                        return_image.size_y = string_type_image_size.parse::<u32>().unwrap();
                        useful_line_number = 2;
                    }
                }
            }
            if useful_line_number == 3 {
                white_value = line_text.parse::<f32>().unwrap();
            }
            if useful_line_number > 3 {
                value_array.push(line_text.parse::<f32>().unwrap());
            }
            useful_line_number += 1;
        }
    }
    return_image.pixel_array =
        vec![vec![0.0; return_image.size_y as usize]; return_image.size_x as usize];
    for pixel_number in 0..return_image.size_y * return_image.size_x {
        return_image.pixel_array[(pixel_number % return_image.size_x) as usize]
            [(pixel_number / return_image.size_x) as usize] =
            value_array[pixel_number as usize] / white_value;
    }
    return_image
}

fn convert_to_pgm(file_name: String) -> String {
    let output_file_name = format!("{}_output.pgm", file_name);
    let source_image_path = Path::new(&file_name);
    let target_image_path = Path::new(&output_file_name);
    let config = PGMConfig::new();
    let input = ImageResource::from_path(source_image_path);
    let mut output = ImageResource::from_path(target_image_path);
    to_pgm(&mut output, &input, &config).unwrap();
    output_file_name
}

fn parse_binary_pgm(binary_path: String) -> String {
    let file_vec = read_a_file(binary_path);
    let mut return_string = "".to_string();
    let mut line_number = 1u32;
    for element_i in file_vec.iter() {
        if line_number < 4 {
            return_string.push(std::char::from_u32(*element_i as u32).unwrap());
            if *element_i == 0x0A {
                line_number += 1;
            }
        } else {
            return_string.push_str(&(format!("{}\n", *element_i) as String));
        }
    }
    return_string
}
