use std::str::FromStr;
use std::{io::Read, option::Option};
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use std::fs::File;
use std::io::{ self, BufRead, BufReader, Write };
use serde::{ Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    host: String,
    port: u32,
    user: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TransFile {
    source: String,
    destination: String,
    commands: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Configure {
    pub server: Option<Server>,
    files: Option<Vec<TransFile>>,
    #[serde(rename = "pre-commands")]
    pre_commands: Option<Vec<String>>,
    #[serde(rename = "post-commands")]
    post_commands: Option<Vec<String>>,
}

const PROGRESS_UPDATE_INTERVAL: usize = 1024 * 1024;


pub fn deploy(config: Configure) {
    let server = &config.server.unwrap();
    let addr = format!("{}:{}", server.host, server.port);
    let tcp = TcpStream::connect(addr).unwrap();

    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    //sess.userauth_agent(&server.user).unwrap();
    sess.userauth_password(&server.user, &server.password).unwrap();
    assert!(sess.authenticated());

    for command in config.pre_commands.unwrap_or(Vec::new()) {
        exec_command(&sess, &command).unwrap();
    }
    
    for ele in config.files.unwrap_or(Vec::new()) {
        let local_file = ele.source;
        let remote_path = upload_file(&sess, &local_file, &ele.destination).unwrap();
        println!("remote path: {}", remote_path);
        for command in ele.commands.unwrap_or(Vec::new()) {
            let result = exec_command(&sess, &command).unwrap();
            print!("{}", result);
        }
    }

    for command in config.post_commands.unwrap_or(Vec::new()) {
        exec_command(&sess, &command).unwrap();
    }


}

fn upload_file<'a>(session: &Session, file_path: &str, dest: &str) -> Result<String, io::Error>{
    let mut dest = String::from_str(dest).unwrap();
    let mut local_file = File::open(file_path)?;
    let mut buffer = Vec::new();
    let file_size = local_file.read_to_end(&mut buffer).unwrap();
    let path = Path::new(file_path);
    let file_name = path.file_name().unwrap();
    println!("file size: {}", file_size);

    dest = if dest.contains("~/") {
        let result = exec_command(&session, "pwd").unwrap();
        let r = dest.replace("~/", &format!("{}/", &result.trim()));
        r
    } else {
        dest
    };
    println!("dest: {}", dest);
    let remote_path_str = format!("{}{}", dest, file_name.to_string_lossy());
    let remote_path = Path::new(&remote_path_str);

    println!("remote path: {}", remote_path_str);

    let mut remote_file = session.scp_send(remote_path, 0o644, file_size as u64, None).unwrap();

    println!("start uploading file {}.", file_path);
    for (_, chunk) in buffer.chunks(PROGRESS_UPDATE_INTERVAL).enumerate() {
        let mut send_bytes = 0;
        while send_bytes < chunk.len() {
            let result = remote_file.write(chunk).unwrap();
            send_bytes += result;
        }
    }
    remote_file.send_eof().unwrap();
    println!("successed uploading.");
    Ok(remote_path_str)
}

fn exec_command(session: &Session, command: &str) -> Result<String, ssh2::Error> {
    let mut channel = session.channel_session().unwrap();
    channel.exec(command).unwrap();

    let mut reader = channel.stream(0);
    let mut result = String::new();
    let mut buf = [0; 1024];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let s = String::from_utf8_lossy(&buf[..n]);
                result.push_str(&s);
                println!("{}", s)
            },
            Err(_) => return Err(ssh2::Error::new(ssh2::ErrorCode::Session(0), "")),
        }
    }

    // channel.read_to_string( &mut result).unwrap();
    channel.wait_close()?;
    Ok(result)
}

// fn get_file_size(file: &str) -> u64 {
//     std::fs::metadata(file).map(|metadata| metadata.len()).unwrap_or(0)
// }
