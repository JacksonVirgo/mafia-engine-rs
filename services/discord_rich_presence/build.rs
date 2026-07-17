use std::{
    env,
    error::Error,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

const ICON_PNG: &[u8] = include_bytes!("assets/icon.png");
const ICON_SIZES: &[u32] = &[16, 20, 24, 32, 40, 48, 64, 128, 256];

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.png");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set by cargo"));
        let icon_path =
            write_windows_icon(&out_dir).expect("failed to generate the Windows executable icon");
        let rc_path = write_resource_script(&out_dir, &icon_path)
            .expect("failed to write the Windows resource script");

        embed_resource::compile(&rc_path, embed_resource::NONE)
            .manifest_required()
            .expect("failed to compile the Windows executable resources");
    }
}

fn write_resource_script(out_dir: &Path, icon_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let icon_path = icon_path.to_string_lossy().replace('\\', "\\\\");
    let script = format!(
        "#pragma code_page(65001)\n\
         1 ICON \"{icon_path}\"\n\
         1 VERSIONINFO\n\
         FILEVERSION 0,1,0,0\n\
         PRODUCTVERSION 0,1,0,0\n\
         FILEOS 0x40004\n\
         FILETYPE 0x1\n\
         {{\n\
         BLOCK \"StringFileInfo\"\n\
         {{\n\
         BLOCK \"000004b0\"\n\
         {{\n\
         VALUE \"FileDescription\", \"Discord Mafia Rich Presence\"\n\
         VALUE \"FileVersion\", \"0.1.0\"\n\
         VALUE \"OriginalFilename\", \"mafia-discord-rich-presence.exe\"\n\
         VALUE \"ProductName\", \"Discord Mafia\"\n\
         VALUE \"ProductVersion\", \"0.1.0\"\n\
         }}\n\
         }}\n\
         BLOCK \"VarFileInfo\"\n\
         {{\n\
         VALUE \"Translation\", 0x0, 0x04b0\n\
         }}\n\
         }}\n"
    );
    let path = out_dir.join("resource.rc");
    fs::write(&path, script)?;
    Ok(path)
}

fn write_windows_icon(out_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let decoder = png::Decoder::new(Cursor::new(ICON_PNG));
    let mut reader = decoder.read_info()?;
    let mut source = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or("invalid icon image size")?
    ];
    let image = reader.next_frame(&mut source)?;
    if image.color_type != png::ColorType::Rgba {
        return Err("the executable icon must use RGBA pixels".into());
    }
    source.truncate(image.buffer_size());

    let frames = ICON_SIZES
        .iter()
        .map(|&size| {
            let pixels = resize_rgba(&source, image.width, image.height, size);
            Ok::<_, Box<dyn Error>>((size, encode_png(&pixels, size)?))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut icon = Vec::with_capacity(
        6 + frames.len() * 16 + frames.iter().map(|(_, png)| png.len()).sum::<usize>(),
    );
    icon.extend_from_slice(&0u16.to_le_bytes());
    icon.extend_from_slice(&1u16.to_le_bytes());
    icon.extend_from_slice(&(frames.len() as u16).to_le_bytes());

    let mut offset = 6 + frames.len() * 16;
    for (size, png) in &frames {
        icon.push(if *size == 256 { 0 } else { *size as u8 });
        icon.push(if *size == 256 { 0 } else { *size as u8 });
        icon.extend_from_slice(&[0, 0]);
        icon.extend_from_slice(&1u16.to_le_bytes());
        icon.extend_from_slice(&32u16.to_le_bytes());
        icon.extend_from_slice(&(png.len() as u32).to_le_bytes());
        icon.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += png.len();
    }
    for (_, png) in frames {
        icon.extend_from_slice(&png);
    }

    let path = out_dir.join("discord-mafia.ico");
    fs::write(&path, icon)?;
    Ok(path)
}

fn resize_rgba(source: &[u8], width: u32, height: u32, target_size: u32) -> Vec<u8> {
    let mut target = vec![0; (target_size * target_size * 4) as usize];
    for target_y in 0..target_size {
        let source_y = target_y * height / target_size;
        for target_x in 0..target_size {
            let source_x = target_x * width / target_size;
            let source_offset = ((source_y * width + source_x) * 4) as usize;
            let target_offset = ((target_y * target_size + target_x) * 4) as usize;
            target[target_offset..target_offset + 4]
                .copy_from_slice(&source[source_offset..source_offset + 4]);
        }
    }
    target
}

fn encode_png(pixels: &[u8], size: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut output = Vec::new();
    let mut encoder = png::Encoder::new(&mut output, size, size);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(pixels)?;
    drop(writer);
    Ok(output)
}
