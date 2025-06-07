use gio::prelude::FileExt;
use rusty_tesseract::{image_to_data, Args, DataOutput, Image};

pub fn recognize_text_from_image(file: &gio::File) -> anyhow::Result<DataOutput>{
    let file_path = file.path().unwrap();
    let args = Args::default();
    let img = Image::from_path(file_path)?;
    Ok(image_to_data(&img, &args)?)
}