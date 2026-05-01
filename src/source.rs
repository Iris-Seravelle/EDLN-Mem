use std::fs::File;
use std::os::unix::fs::FileExt;

pub trait DataSource {
    fn fetch_page(&self, addr: usize, buffer: &mut [u8]) -> Result<(), String>;
}

#[allow(dead_code)]
pub struct MockSource;

impl DataSource for MockSource {
    fn fetch_page(&self, addr: usize, buffer: &mut [u8]) -> Result<(), String> {
        // Populate with a predictable pattern: [offset_byte, offset_byte, ...]
        // We use the address to make it unique per page
        let fill_byte = (addr & 0xFF) as u8;
        for byte in buffer.iter_mut() {
            *byte = fill_byte;
        }

        Ok(())
    }
}

pub struct FileSource {
    file: File,
    file_len: u64,
}

impl FileSource {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        Ok(FileSource {
            file,
            file_len: metadata.len(),
        })
    }
}

impl DataSource for FileSource {
    fn fetch_page(&self, addr: usize, buffer: &mut [u8]) -> Result<(), String> {
        if self.file_len == 0 {
            return Err("Backing file is empty".into());
        }

        // Use the address to calculate an offset into the file (wrapping if necessary)
        let offset = (addr as u64) % self.file_len;

        // pwrite/pread equivalent in Rust: read_at
        self.file
            .read_at(buffer, offset)
            .map_err(|e| format!("File read failed: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_source_fetch() {
        let source = MockSource;
        let mut buffer = vec![0u8; 4096];
        source.fetch_page(0x12345678, &mut buffer).unwrap();
        assert_eq!(buffer[0], 0x78);
        assert_eq!(buffer[4095], 0x78);
    }
}
