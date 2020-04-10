use std::str;

pub fn handle_req(rreq: &[u8]) {
    let req : Vec<&[u8]> = rreq.split(|x| *x == 0).filter(|x| !x.is_empty()).collect();
    if req.is_empty() { return; }
    let op = str::from_utf8(&req[0]).unwrap().to_owned();
    println!("handing {:?}", req);
    match op.as_str() {
        "create_dir" => {
            println!("create_dir");
        },
        "copy" => {
            println!("copy");
        },
        "write" => {},
        "read" => {},
        _ => {}
    }


}
