#[derive(Debug, Clone, PartialEq)]
pub struct VicBlob {
    pub data: Vec<u8>,
    pub length: u64,
    pub data_type: String,
    pub hash: String,
}

impl VicBlob {
    pub fn new(data: Vec<u8>, length: u64, data_type: String, hash: String) -> VicBlob {
        VicBlob {
            data,
            length,
            data_type,
            hash,
        }
    }

    pub fn new_empty() -> VicBlob {
        VicBlob {
            data: Vec::new(),
            length: 0,
            data_type: String::from(""),
            hash: String::from(""),
        }
    }
}

impl Default for VicBlob {
    fn default() -> VicBlob {
        VicBlob::new_empty()
    }
}
