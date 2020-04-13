use std::{fs, str};

pub fn handle_req(req: Vec<&[u8]>) -> std::io::Result<Vec<u8>> {
    let op = str::from_utf8(&req[0]).unwrap().to_owned();
    match op.as_str() {
        "create_dir" => {
            let path = str::from_utf8(&req[1]).unwrap();
            println!("creating dir named: {:?}", path);
            fs::create_dir(path)?;
            Ok(vec![])
        },
        "copy" => {
            let from = str::from_utf8(&req[1]).unwrap();
            let to = str::from_utf8(&req[2]).unwrap();
            println!("copying from {:?} to {:?}", from ,to);
            fs::copy(from, to)?;
            Ok(vec![])
        },
        "write" => {
            let file = str::from_utf8(&req[1]).unwrap();
            let body = &req[2];
            let bodystr = str::from_utf8(&req[2]).unwrap();
            println!("writing to file: {:?}, body: {:?}", file, bodystr);
            fs::write(file, body)?;
            Ok(vec![])
        },
        "read" => {
            let file = str::from_utf8(&req[1]).unwrap();
            println!("reading from file : {:?}", file);
            let body = fs::read(file)?;
            println!("BODY: {:?}", body);
            Ok(body)
        },
        "remove_dir_all" => {
            let path = str::from_utf8(&req[1]).unwrap();
            println!("removing dir: {:?}", path);
            fs::remove_dir_all(path)?;
            Ok(vec![])

        },
        "remove_dir" => {
            let path = str::from_utf8(&req[1]).unwrap();
            println!("removing empty dir: {:?}", path);
            fs::remove_dir(path)?;
            Ok(vec![])
        },
        _ => {
            Ok(b"Op not supported".to_vec())
        }
    }


}
