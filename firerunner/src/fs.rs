use std::{fs, str};

pub fn handle_req(req: Vec<&[u8]>) -> std::io::Result<Vec<u8>> {
    let op = str::from_utf8(&req[0]).unwrap().to_owned();
    println!("handing {:?}", req);
    match op.as_str() {
        "create_dir" => {
            println!("creating dir");
            let path = str::from_utf8(&req[1]).unwrap();
            fs::create_dir(path)?;
            Ok(vec![1])
        },
        "copy" => {
            println!("copying");
            let from = str::from_utf8(&req[1]).unwrap();
            let to = str::from_utf8(&req[2]).unwrap();
            fs::copy(from, to)?;
            Ok(vec![1])
        },
        "write" => {
            println!("writing to file");
            let file = str::from_utf8(&req[1]).unwrap();
            let body = &req[2];
            fs::write(file, body)?;
            Ok(vec![1])
        },
        "read" => {
            let file = str::from_utf8(&req[1]).unwrap();
            println!("reading from file : {:?}", file);
            let body = fs::read(file)?;
            println!("BODY: {:?}", body);
            Ok(body)
        },
        "remove_dir_all" => {
            println!("removing dir");
            let path = str::from_utf8(&req[1]).unwrap();
            fs::remove_dir_all(path)?;
            Ok(vec![1])

        },
        _ => {
            Ok(vec![0])
        }
    }


}
