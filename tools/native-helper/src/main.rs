mod assets;
mod windows;

use std::path::Path;

fn run(args: &[String]) -> Result<(), (i32, String)> {
    let Some(command) = args.first() else {
        return Err((2, "Usage: quickpick-native-helper png-to-bmp <input.png> <output.bmp> [width height] | verify-hotkeys | capture-readme".into()));
    };
    match command.as_str() {
        "png-to-bmp" => match &args[1..] {
            [input, output] => {
                assets::png_to_bmp(Path::new(input), Path::new(output)).map_err(|error| (1, error))
            }
            [input, output, width, height] => {
                let dimensions = width
                    .parse::<u32>()
                    .ok()
                    .zip(height.parse::<u32>().ok())
                    .filter(|(width, height)| *width > 0 && *height > 0)
                    .ok_or_else(|| (1, "Width and height must be positive integers".into()))?;
                assets::png_to_bmp_cropped(
                    Path::new(input),
                    Path::new(output),
                    dimensions.0,
                    dimensions.1,
                )
                .map_err(|error| (1, error))
            }
            _ => Err((
                1,
                "Usage: png-to-bmp <input.png> <output.bmp> [width height]".into(),
            )),
        },
        "verify-hotkeys" | "capture-readme" => {
            windows::run(args).map_err(|error| (error.code, error.message))
        }
        _ => Err((1, "Unknown native helper command".into())),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err((code, message)) = run(&args) {
        eprintln!("{message}");
        std::process::exit(code);
    }
}
