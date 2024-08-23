use std::io::Write;
use std::string::FromUtf8Error;
use bytebuffer::ByteBuffer;
use bytebuffer::Endian;

#[derive(Debug, Default)]
pub struct ByteBuff {
    pub multiplier: f64,
    pub endian: String,
    pub buffer: ByteBuffer,
    pub total_buffer_length: i64,
}

impl ByteBuff {
    pub fn init(&mut self, endian: String) {
        if endian == "big" {
            self.buffer.set_endian(Endian::BigEndian);
        } else {
            self.buffer.set_endian(Endian::LittleEndian);
        }

        if self.multiplier == 0.0 {
            self.multiplier = 10000.0;
        }
    }

    pub fn wrap(&mut self, byte_data: Vec<u8>){

        self.buffer = ByteBuffer::from_vec(byte_data);
        self.total_buffer_length = self.buffer.len() as i64;
    }

    pub fn put_short(&mut self, value: i16) {
        self.buffer.write_u16(value as u16);
    }

    pub fn put_int(&mut self, value: i32) {
        self.buffer.write_u32(value as u32);
    }

    pub fn put_long(&mut self, value: i64) {
        self.buffer.write_u64(value as u64);
    }

    pub fn put_float(&mut self, value: f64) {
        self.buffer.write_u64((value * self.multiplier) as u64);
    }

    pub fn put_bool(&mut self, value: bool) {
        if value {
            self.buffer.write_u8(1);
        } else {
            self.buffer.write_u8(0);
        }
    }

    pub fn get(&mut self) -> Vec<u8> {

        return match self.buffer.read_u64() {
            Ok(value) => {
                let total_length = value as usize;
                match self.buffer.read_bytes(total_length){
                    Ok(val) => {
                        return val;
                    }
                    Err(err) => {
                        println!("{:?}", err);
                        vec![]
                    }
                }
            }
            Err(err) => {
                println!("{:?}", err);
                vec![]
            }
        }
    }

    pub fn put(&mut self, value: Vec<u8>) {
        self.buffer.write_u64(value.len() as u64);
        self.buffer.write(value.as_slice()).expect("failed to write");
    }

    pub fn put_string(&mut self, value: String) {
        let str_len: i64 = value.len() as i64;
        if str_len > 0 {
            if str_len < 128 {
                self.buffer.write_u8(1);
                self.buffer.write_u8(str_len as u8);
            } else if str_len < 32768 {
                self.buffer.write_u8(2);
                self.buffer.write_u16(str_len as u16);
            } else if str_len < 2147483648 {
                self.buffer.write_u8(3);
                self.buffer.write_u32(str_len as u32);
            } else {
                self.buffer.write_u8(4);
                self.buffer.write_u64(str_len as u64);
            }

            self.buffer.write_bytes(value.as_bytes());
        } else {
            self.buffer.write_u8(1);
            self.buffer.write_u8(1);
            self.buffer.write_bytes("X".as_bytes());
        }
    }

    pub fn get_short(&mut self) -> i16 {
        return match self.buffer.read_u16() {
            Ok(value) => {
                value as i16
            }
            Err(err) => {
                println!("{:?}", err);
                0
            }
        }
    }

    pub fn get_int(&mut self) -> i32 {
        return match self.buffer.read_u32() {
            Ok(value) => {
                value as i32
            }
            Err(err) => {
                println!("{:?}", err);
                0
            }
        }
    }

    pub fn get_long(&mut self) -> i64 {
        return match self.buffer.read_u64() {
            Ok(value) => {
                value as i64
            }
            Err(err) => {
                println!("{:?}", err);
                0
            }
        }
    }

    pub fn get_float(&mut self) -> f64 {
        return match self.buffer.read_u64() {
            Ok(value) => {
                value as f64 / self.multiplier
            }
            Err(err) => {
                println!("{:?}", err);
                0.0
            }
        }
    }

    pub fn get_bool(&mut self) -> bool {
        return match self.buffer.read_u8() {
            Ok(value) => {
                if value == 1 {
                    true
                } else {
                    false
                }
            }
            Err(err) => {
                println!("{:?}", err);
                false
            }
        }
    }

    pub fn get_string(&mut self) -> String {
        return match self.buffer.read_u8() {
            Ok(type_string) => {
                let mut string_data = String::from("");

                if type_string == 1 {
                    match self.buffer.read_u8(){
                        Ok(str_len) => {

                            match self.buffer.read_bytes(str_len as usize){
                                Ok(string_val) => {
                                    match String::from_utf8(string_val){
                                        Ok(val) => {
                                            string_data = val;
                                        }
                                        Err(err) => {
                                            println!("{:?}", err);
                                            return "".to_string();
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }

                } else if type_string == 2 {
                    match self.buffer.read_u16(){
                        Ok(str_len) => {

                            match self.buffer.read_bytes(str_len as usize){
                                Ok(string_val) => {
                                    match String::from_utf8(string_val){
                                        Ok(val) => {
                                            string_data = val;
                                        }
                                        Err(err) => {
                                            println!("{:?}", err);
                                            return "".to_string();
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                } else if type_string == 3 {
                    match self.buffer.read_u32(){
                        Ok(str_len) => {

                            match self.buffer.read_bytes(str_len as usize){
                                Ok(string_val) => {
                                    match String::from_utf8(string_val){
                                        Ok(val) => {
                                            string_data = val;
                                        }
                                        Err(err) => {
                                            println!("{:?}", err);
                                            return "".to_string();
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                } else {
                    match self.buffer.read_u64(){
                        Ok(str_len) => {

                            match self.buffer.read_bytes(str_len as usize){
                                Ok(string_val) => {
                                    match String::from_utf8(string_val){
                                        Ok(val) => {
                                            string_data = val;
                                        }
                                        Err(err) => {
                                            println!("{:?}", err);
                                            return "".to_string();
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }

                if string_data == "X" {
                    "".to_string()
                } else {
                    string_data
                }
            }
            Err(err) => {
                println!("{:?}", err);
                "".to_string()
            }
        }
    }

    pub fn to_array(&self) -> Vec<u8> {
        return self.buffer.to_owned().into_vec();
    }
}
