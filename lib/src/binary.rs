use crate::credentials::Credentials;
use crate::region::Region;
use serde_json::Value as JsonValue;
use std::io::prelude::*;
use std::io::Read;
use std::net::TcpStream;

pub(crate) fn build_buffer(len: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(len);
    (0..len).for_each(|_| res.push(0));
    res
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut buffer: [u8; 8] = [0; 8];
    buffer[..bytes.len()].clone_from_slice(bytes);
    u64::from_le_bytes(buffer)
}

struct BinaryReader {
    buffer: Vec<u8>,
    offset: usize,
}

impl BinaryReader {
    fn new(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut length: [u8; 4] = [0; 4];
        reader.read_exact(&mut length).map_err(Error::read)?;
        let length = u32::from_le_bytes(length) as usize;
        let mut buffer = build_buffer(length);
        reader.read_exact(&mut buffer).map_err(Error::read)?;
        Ok(Self { buffer, offset: 0 })
    }

    fn get_byte(&mut self) -> Option<u8> {
        let value = self.peek_byte();
        self.offset += 1;
        value
    }

    fn peek_byte(&mut self) -> Option<u8> {
        self.buffer.get(self.offset).copied()
    }

    fn get_bytes(&mut self, cnt: usize) -> Vec<u8> {
        let result = self.buffer[self.offset..(self.offset + cnt)].to_vec();
        self.offset += cnt;
        result
    }

    fn parse_array(&mut self, cache: &mut Vec<String>) -> Result<JsonValue, Error> {
        let mut res = Vec::new();
        while let Some(ftype) = self.get_byte() {
            if ftype == 255 {
                break;
            }
            res.push(self.parse_type(cache, ftype)?);
        }
        Ok(JsonValue::Array(res))
    }

    fn parse_object(&mut self, cache: &mut Vec<String>) -> Result<JsonValue, Error> {
        let mut res = serde_json::Map::new();
        while let Some(ftype) = self.get_byte() {
            if ftype == 255 {
                break;
            }
            let key = self.parse_text(cache, ftype)?;
            let value = self.run_parse(cache)?;
            res.insert(key, value);
        }
        Ok(JsonValue::Object(res))
    }

    fn parse_text(&mut self, cache: &mut Vec<String>, ftype: u8) -> Result<String, Error> {
        if (100..150).contains(&ftype) || ftype <= 3 {
            self.parse_text_to_cache(cache, ftype)
        } else {
            self.parse_text_from_cache(cache, ftype)
        }
    }

    fn parse_text_to_cache(&mut self, cache: &mut Vec<String>, ftype: u8) -> Result<String, Error> {
        let len = if (100..150).contains(&ftype) {
            (ftype - 100) as u64
        } else {
            let data = self.get_bytes((ftype + 1) as usize);
            bytes_to_u64(&data)
        };
        let data = self.get_bytes(len as usize);
        let res = String::from_utf8(data).map_err(Error::read)?;
        cache.push(res.clone());
        Ok(res)
    }

    fn parse_text_from_cache(
        &mut self,
        cache: &mut Vec<String>,
        ftype: u8,
    ) -> Result<String, Error> {
        let idx = if (150..200).contains(&ftype) {
            (ftype - 150) as usize
        } else {
            let data = self.get_bytes((ftype - 3) as usize);
            bytes_to_u64(&data) as usize
        };
        cache
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::Read(format!("string not found in cache at index {}", idx)))
    }

    fn parse_type(&mut self, cache: &mut Vec<String>, ftype: u8) -> Result<JsonValue, Error> {
        if (8..=15).contains(&ftype) {
            let data = self.get_bytes((ftype - 7) as usize);
            Ok(JsonValue::Number(bytes_to_u64(&data).into()))
        } else if (200..220).contains(&ftype) {
            Ok(JsonValue::Number((ftype - 200).into()))
        } else if (100..200).contains(&ftype) || ftype < 8 {
            self.parse_text(cache, ftype).map(JsonValue::String)
        } else if ftype == 19 {
            Ok(JsonValue::Bool(true))
        } else if ftype == 18 {
            Ok(JsonValue::Bool(false))
        } else if ftype == 17 {
            self.parse_array(cache)
        } else if ftype == 16 {
            self.parse_object(cache)
        } else if ftype == 20 {
            let data = self.get_bytes(8);
            Ok(JsonValue::Number(bytes_to_u64(&data).into()))
        } else {
            println!("ftype {} unimplemented", ftype);
            unimplemented!()
        }
    }

    fn run_parse(&mut self, cache: &mut Vec<String>) -> Result<JsonValue, Error> {
        let ftype = self.get_byte().unwrap();
        self.parse_type(cache, ftype)
    }

    pub fn parse<R: Read>(read: &mut R) -> Result<JsonValue, Error> {
        let mut reader = BinaryReader::new(read)?;
        let mut cache: Vec<String> = Vec::new();
        reader.run_parse(&mut cache)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Text(String),
    Number(u64),
}

impl Value {
    pub fn as_bool(self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_text(self) -> Option<String> {
        if let Self::Text(value) = self {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_number(self) -> Option<u64> {
        if let Self::Number(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

struct CommandBuilder(Vec<u8>);

impl CommandBuilder {
    fn new() -> Self {
        Self(vec![0, 0])
    }

    fn push(&mut self, value: u8) {
        self.0.push(value);
    }

    fn push_u32(&mut self, value: u32) {
        let bytes = value.to_le_bytes();
        for b in bytes {
            self.push(b);
        }
    }

    fn push_u64(&mut self, value: u64) {
        let bytes = value.to_le_bytes();
        for b in bytes {
            self.push(b);
        }
    }

    fn push_str(&mut self, value: &str) {
        for b in value.as_bytes() {
            self.push(*b);
        }
    }

    fn push_lstr(&mut self, value: &str) {
        assert!(value.as_bytes().len() < 255);
        self.push(value.as_bytes().len() as u8);
        self.push_str(value);
    }

    fn push_str_param(&mut self, key: &str, value: &str) {
        self.push_lstr(key);
        self.push_u32(value.as_bytes().len() as u32);
        self.push_str(value);
    }

    fn push_bool_param(&mut self, key: &str, value: bool) {
        self.push((key.as_bytes().len() as u8) + 0x80);
        self.push_str(key);
        self.push(if value { 1 } else { 0 });
    }

    fn push_number_param(&mut self, key: &str, value: u64) {
        self.push((key.as_bytes().len() as u8) + 0x40);
        self.push_str(key);
        self.push_u64(value);
    }

    fn build(mut self) -> Vec<u8> {
        let size = self.0.len();
        self.0[0] = ((size - 2) % 256) as u8;
        self.0[1] = ((size - 2) / 256) as u8;
        self.0
    }
}

#[derive(Debug)]
pub enum Error {
    Connection(String),
    Read(String),
    Write(String),
}

impl Error {
    pub fn connection<T: ToString>(input: T) -> Self {
        Self::Connection(input.to_string())
    }
    pub fn read<T: ToString>(input: T) -> Self {
        Self::Read(input.to_string())
    }
    pub fn write<T: ToString>(input: T) -> Self {
        Self::Write(input.to_string())
    }
}

pub struct BinaryClient {
    pub(crate) stream: TcpStream,
    credentials: Credentials,
}

impl BinaryClient {
    pub fn new(region: Region, credentials: Credentials) -> Result<Self, Error> {
        let address = format!("{}:{}", region.address(), 8398);
        Ok(Self {
            stream: TcpStream::connect(address).map_err(Error::connection)?,
            credentials,
        })
    }

    pub fn from_env() -> Result<Self, Error> {
        Self::new(Region::from_env(), Credentials::from_env())
    }

    fn read_result(&mut self) -> Result<JsonValue, Error> {
        BinaryReader::parse(&mut self.stream)
    }

    fn build_command(
        method: &str,
        params: &[(&str, Value)],
        has_data: bool,
        _data_len: usize,
    ) -> Vec<u8> {
        let mut cmd = CommandBuilder::new();
        if has_data {
            unimplemented!()
        } else {
            cmd.push_lstr(method);
        }
        cmd.push(params.len() as u8);
        for (key, value) in params.iter() {
            match value {
                Value::Text(value) => cmd.push_str_param(key, value.as_str()),
                Value::Bool(value) => cmd.push_bool_param(key, *value),
                Value::Number(value) => cmd.push_number_param(key, *value),
            }
        }
        cmd.build()
    }

    pub fn send_command(
        &mut self,
        method: &str,
        params: &[(&str, Value)],
        has_data: bool,
        data_len: usize,
    ) -> Result<JsonValue, Error> {
        let mut creds = self.credentials.to_binary_params();
        creds.extend_from_slice(params);
        let cmd = Self::build_command(method, &creds, has_data, data_len);
        let count = self.stream.write(&cmd).map_err(Error::write)?;
        assert_eq!(count, cmd.len());
        self.read_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::Credentials;
    use crate::region::Region;

    #[tokio::test]
    async fn execute_list_root() {
        let creds = Credentials::from_env();
        let mut protocol = BinaryClient::new(Region::Europe, creds).unwrap();
        let params: Vec<(&str, Value)> = vec![("folderid", Value::Number(0))];
        let result = protocol
            .send_command("listfolder", &params, false, 0)
            .unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn parse_result() {
        let input: Vec<u8> = vec![
            0x58, 0x02, 0x00, 0x00, 0x10, 0x6A, 0x72, 0x65, 0x73, 0x75, 0x6C, 0x74, 0xC8, 0x6C,
            0x6D, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x10, 0x68, 0x70, 0x61, 0x74, 0x68,
            0x65, 0x2F, 0x68, 0x6E, 0x61, 0x6D, 0x65, 0x99, 0x6B, 0x63, 0x72, 0x65, 0x61, 0x74,
            0x65, 0x64, 0x83, 0x46, 0x72, 0x69, 0x2C, 0x20, 0x31, 0x36, 0x20, 0x4A, 0x75, 0x6C,
            0x20, 0x32, 0x30, 0x32, 0x31, 0x20, 0x31, 0x32, 0x3A, 0x31, 0x35, 0x3A, 0x34, 0x35,
            0x20, 0x2B, 0x30, 0x30, 0x30, 0x30, 0x6A, 0x69, 0x73, 0x6D, 0x69, 0x6E, 0x65, 0x13,
            0x69, 0x74, 0x68, 0x75, 0x6D, 0x62, 0x12, 0x6C, 0x6D, 0x6F, 0x64, 0x69, 0x66, 0x69,
            0x65, 0x64, 0x9C, 0x66, 0x69, 0x64, 0x66, 0x64, 0x30, 0x6C, 0x63, 0x6F, 0x6E, 0x74,
            0x65, 0x6E, 0x74, 0x73, 0x11, 0x10, 0x98, 0x6A, 0x2F, 0x42, 0x6F, 0x6F, 0x6B, 0x73,
            0x9A, 0x69, 0x42, 0x6F, 0x6F, 0x6B, 0x73, 0x9B, 0x83, 0x46, 0x72, 0x69, 0x2C, 0x20,
            0x31, 0x36, 0x20, 0x4A, 0x75, 0x6C, 0x20, 0x32, 0x30, 0x32, 0x31, 0x20, 0x31, 0x32,
            0x3A, 0x33, 0x35, 0x3A, 0x33, 0x37, 0x20, 0x2B, 0x30, 0x30, 0x30, 0x30, 0x9D, 0x13,
            0x9E, 0x12, 0x9F, 0xA5, 0x6C, 0x63, 0x6F, 0x6D, 0x6D, 0x65, 0x6E, 0x74, 0x73, 0xC8,
            0xA0, 0x6F, 0x64, 0x31, 0x30, 0x34, 0x31, 0x37, 0x33, 0x33, 0x39, 0x33, 0x33, 0x6C,
            0x69, 0x73, 0x73, 0x68, 0x61, 0x72, 0x65, 0x64, 0x12, 0x68, 0x69, 0x63, 0x6F, 0x6E,
            0x6A, 0x66, 0x6F, 0x6C, 0x64, 0x65, 0x72, 0x6C, 0x69, 0x73, 0x66, 0x6F, 0x6C, 0x64,
            0x65, 0x72, 0x13, 0x72, 0x70, 0x61, 0x72, 0x65, 0x6E, 0x74, 0x66, 0x6F, 0x6C, 0x64,
            0x65, 0x72, 0x69, 0x64, 0xC8, 0x6C, 0x66, 0x6F, 0x6C, 0x64, 0x65, 0x72, 0x69, 0x64,
            0x0B, 0x2D, 0x99, 0x17, 0x3E, 0xFF, 0x10, 0x98, 0x6E, 0x2F, 0x44, 0x6F, 0x63, 0x75,
            0x6D, 0x65, 0x6E, 0x74, 0x73, 0x9A, 0x6D, 0x44, 0x6F, 0x63, 0x75, 0x6D, 0x65, 0x6E,
            0x74, 0x73, 0x9B, 0x83, 0x46, 0x72, 0x69, 0x2C, 0x20, 0x31, 0x36, 0x20, 0x4A, 0x75,
            0x6C, 0x20, 0x32, 0x30, 0x32, 0x31, 0x20, 0x31, 0x32, 0x3A, 0x33, 0x32, 0x3A, 0x35,
            0x39, 0x20, 0x2B, 0x30, 0x30, 0x30, 0x30, 0x9D, 0x13, 0x9E, 0x12, 0x9F, 0x83, 0x46,
            0x72, 0x69, 0x2C, 0x20, 0x31, 0x36, 0x20, 0x4A, 0x75, 0x6C, 0x20, 0x32, 0x30, 0x32,
            0x31, 0x20, 0x31, 0x32, 0x3A, 0x33, 0x33, 0x3A, 0x31, 0x35, 0x20, 0x2B, 0x30, 0x30,
            0x30, 0x30, 0xA6, 0xC8, 0xA0, 0x6F, 0x64, 0x31, 0x30, 0x34, 0x31, 0x37, 0x32, 0x38,
            0x30, 0x38, 0x30, 0xA8, 0x12, 0xA9, 0xAA, 0xAB, 0x13, 0xAC, 0xC8, 0xAD, 0x0B, 0x50,
            0x82, 0x17, 0x3E, 0xFF, 0x10, 0x98, 0x6D, 0x2F, 0x50, 0x69, 0x63, 0x74, 0x75, 0x72,
            0x65, 0x73, 0x9A, 0x6C, 0x50, 0x69, 0x63, 0x74, 0x75, 0x72, 0x65, 0x73, 0x9B, 0x9C,
            0x9D, 0x13, 0x9E, 0x12, 0x9F, 0x83, 0x46, 0x72, 0x69, 0x2C, 0x20, 0x31, 0x36, 0x20,
            0x4A, 0x75, 0x6C, 0x20, 0x32, 0x30, 0x32, 0x31, 0x20, 0x32, 0x30, 0x3A, 0x33, 0x38,
            0x3A, 0x34, 0x36, 0x20, 0x2B, 0x30, 0x30, 0x30, 0x30, 0xA6, 0xC8, 0xA0, 0x6F, 0x64,
            0x31, 0x30, 0x34, 0x31, 0x36, 0x37, 0x38, 0x37, 0x32, 0x37, 0xA8, 0x12, 0xA9, 0xAA,
            0xAB, 0x13, 0xAC, 0xC8, 0xAD, 0x0B, 0x87, 0xC1, 0x16, 0x3E, 0xFF, 0x10, 0x98, 0x6C,
            0x2F, 0x54, 0x56, 0x53, 0x68, 0x6F, 0x77, 0x73, 0x9A, 0x6B, 0x54, 0x56, 0x53, 0x68,
            0x6F, 0x77, 0x73, 0x9B, 0x83, 0x4D, 0x6F, 0x6E, 0x2C, 0x20, 0x31, 0x39, 0x20, 0x4A,
            0x75, 0x6C, 0x20, 0x32, 0x30, 0x32, 0x31, 0x20, 0x30, 0x36, 0x3A, 0x30, 0x34, 0x3A,
            0x33, 0x31, 0x20, 0x2B, 0x30, 0x30, 0x30, 0x30, 0x9D, 0x13, 0x9E, 0x12, 0x9F, 0x83,
            0x4D, 0x6F, 0x6E, 0x2C, 0x20, 0x31, 0x39, 0x20, 0x4A, 0x75, 0x6C, 0x20, 0x32, 0x30,
            0x32, 0x31, 0x20, 0x30, 0x36, 0x3A, 0x30, 0x35, 0x3A, 0x30, 0x30, 0x20, 0x2B, 0x30,
            0x30, 0x30, 0x30, 0xA6, 0xC8, 0xA0, 0x6F, 0x64, 0x31, 0x30, 0x35, 0x33, 0x36, 0x30,
            0x33, 0x31, 0x32, 0x39, 0xA8, 0x12, 0xA9, 0xAA, 0xAB, 0x13, 0xAC, 0xC8, 0xAD, 0x0B,
            0x39, 0xB5, 0xCC, 0x3E, 0xFF, 0xFF, 0xA8, 0x12, 0xA9, 0xAA, 0xAB, 0x13, 0xAD, 0xC8,
            0xFF, 0xFF,
        ];
        let mut reader = input.as_slice();
        let res = BinaryReader::parse(&mut reader).unwrap();
        let res = res.as_object().unwrap();
        assert_eq!(
            res.get("result").and_then(|item| item.clone().as_u64()),
            Some(0)
        );
    }

    #[test]
    fn build_command_number() {
        let params: Vec<(&str, Value)> = vec![("folderid".into(), Value::Number(0))];
        let result = BinaryClient::build_command("listfolder", &params, false, 0);
        let expected: Vec<u8> = vec![
            0x1D, 0x00, 0x0A, 0x6C, 0x69, 0x73, 0x74, 0x66, 0x6F, 0x6C, 0x64, 0x65, 0x72, 0x01,
            0x48, 0x66, 0x6F, 0x6C, 0x64, 0x65, 0x72, 0x69, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn build_command_text() {
        let params: Vec<(&str, Value)> = vec![(
            "auth".into(),
            Value::Text("Ec7QkEjFUnzZ7Z8W2YH1qLgxY7gGvTe09AH0i7V3kX".into()),
        )];
        let result = BinaryClient::build_command("listfolder", &params, false, 0);
        let expected: Vec<u8> = vec![
            0x3F, 0x00, 0x0A, 0x6C, 0x69, 0x73, 0x74, 0x66, 0x6F, 0x6C, 0x64, 0x65, 0x72, 0x01,
            0x04, 0x61, 0x75, 0x74, 0x68, 0x2A, 0x00, 0x00, 0x00, 0x45, 0x63, 0x37, 0x51, 0x6B,
            0x45, 0x6A, 0x46, 0x55, 0x6E, 0x7A, 0x5A, 0x37, 0x5A, 0x38, 0x57, 0x32, 0x59, 0x48,
            0x31, 0x71, 0x4C, 0x67, 0x78, 0x59, 0x37, 0x67, 0x47, 0x76, 0x54, 0x65, 0x30, 0x39,
            0x41, 0x48, 0x30, 0x69, 0x37, 0x56, 0x33, 0x6B, 0x58,
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn build_command_multiple() {
        let params: Vec<(&str, Value)> = vec![
            (
                "auth".into(),
                Value::Text("Ec7QkEjFUnzZ7Z8W2YH1qLgxY7gGvTe09AH0i7V3kX".into()),
            ),
            ("folderid".into(), Value::Number(0)),
        ];
        let result = BinaryClient::build_command("listfolder", &params, false, 0);
        let expected: Vec<u8> = vec![
            0x50, 0x00, 0x0A, 0x6C, 0x69, 0x73, 0x74, 0x66, 0x6F, 0x6C, 0x64, 0x65, 0x72, 0x02,
            0x04, 0x61, 0x75, 0x74, 0x68, 0x2A, 0x00, 0x00, 0x00, 0x45, 0x63, 0x37, 0x51, 0x6B,
            0x45, 0x6A, 0x46, 0x55, 0x6E, 0x7A, 0x5A, 0x37, 0x5A, 0x38, 0x57, 0x32, 0x59, 0x48,
            0x31, 0x71, 0x4C, 0x67, 0x78, 0x59, 0x37, 0x67, 0x47, 0x76, 0x54, 0x65, 0x30, 0x39,
            0x41, 0x48, 0x30, 0x69, 0x37, 0x56, 0x33, 0x6B, 0x58, 0x48, 0x66, 0x6F, 0x6C, 0x64,
            0x65, 0x72, 0x69, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(expected, result);
    }
}
