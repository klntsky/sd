use std::collections::HashMap;

pub struct RuntimeMock {
    pub env : HashMap<String, String>,
    pub stdin : Vec<u8>,
    pub files : HashMap<String, String>
}
