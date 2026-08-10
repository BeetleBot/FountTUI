use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn pack(xml: &str) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("document.xml", options)?;
        zip.write_all(xml.as_bytes())?;
        zip.finish()?;
    }
    Ok(buf)
}
