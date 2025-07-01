use exif_edit::exif::{ExifEditData, ExifTime};
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use std::fs;

fn main() {
    let file_path = "static/image/mountain.JPEG";
    let mut file_data = fs::read(file_path).unwrap();
    let v = file_path.split(".").collect::<Vec<&str>>();
    let (name, ext) = (v[0], v[1]);

    let ft = if ext == "JPEG" {
        FileExtension::JPEG
    } else if ext == "png" {
        FileExtension::PNG { as_zTXt_chunk: true }
    } else if ext == "tif" {
        FileExtension::TIFF
    } else {
        FileExtension::WEBP
    };
    let metadata = match Metadata::new_from_vec(&file_data, ft) {
        Ok(metadata) => metadata,
        Err(_) => Metadata::new()
    };
    let exif = ExifEditData::new(&metadata);
    println!("{:?}", exif.others_to_str());
    //exif.update_datetime("1999-01-01T00:00:00", ExifTime::DateTimeOriginal);
    //match exif.metadata.write_to_vec(&mut file_data, ft) {
    //    Ok(_) => {}
    //    Err(e) => { eprintln!("{:?}", e); }
    //};
    //fs::write(name.to_string() + "_meta." + ext, file_data).unwrap();
}