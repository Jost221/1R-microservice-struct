use std::fs::File;
use std::io::{Error, Read, Write};

use crate::json_struct::*;
use crate::config;

pub async fn set_data(data: WriteData) -> Result<String, Error>{
    let path = config::JsonPath::from_env().expect("Failed to load json path");

    return match read_or_create_json_file(path.to_string(), Some(true)) {
        Ok(mut read_data) => {
            let mut mut_flag = false;
            for element in read_data.nods.iter_mut(){
                if element.id != data.id { continue; }

                element.address.push(data.address.clone());
                mut_flag = true;
                break;
            }

            if !mut_flag {
                read_data.nods.push(
                    Node { 
                        id: data.id, 
                        address: [data.address].to_vec() 
                    }
                );
            }
            write_json_file(path.to_string(), read_data)?;
            Ok("Sucess".to_string())
        }
        Err(e) => Err(e)
    }
}

pub async fn get_writed() -> Result<Data, Error>{
    let path = config::JsonPath::from_env().expect("Failed to load json path");

    read_or_create_json_file(path.to_string(), Some(false))
}

pub async fn remove_data_from_file(input_data: WriteData) -> Result<String, Error>{
    let path = config::JsonPath::from_env().expect("Failed to load json path");

    if let Ok(mut data) = read_or_create_json_file(path.to_string(), Some(false)) {
        let mut is_mute = false;
        for node in data.nods.iter_mut() {
            
            if node.id != input_data.id { continue; }
            
            for index in (0..node.address.len()).rev() {
                if node.address[index] == input_data.address {
                    node.address.remove(index);
                    is_mute = true;
                    dbg!(index);
                }
            }
            break;
        }

        if !is_mute {
            return Err(Error::new(std::io::ErrorKind::NotFound, format!("Not found address with set id {}", input_data.id)))
        }

        write_json_file(path.to_string(), data)?;
        Ok("Sucess".to_string())
    } else {
      Err(Error::new(std::io::ErrorKind::NotFound, "Not found file"))
    }
}

fn read_or_create_json_file(file_path: String, need_create: Option<bool>) -> Result<Data, Error> {
    let data = if !std::path::Path::new(file_path.as_str()).exists() && need_create.unwrap_or(true){
        let mut file = File::create(file_path)?;
        let empty_data = Data {
            nods: Vec::new()
        };
        let json = serde_json::to_string(&empty_data)?;
        file.write_all(json.as_bytes())?;
        empty_data
    } else {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    Ok(data)
}

fn write_json_file(file_path: String, data: Data) -> Result<(), Error> {
    let mut file = File::create(file_path)?;
    let json = serde_json::to_string(&data)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}