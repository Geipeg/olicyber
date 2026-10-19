use std::io::{Read, Write};
use std::net::TcpStream;

use anyhow::Result;
use ctf_party::base64::{from_b64, to_b64};
use ctf_party::bin::{from_bin, to_bin};
use ctf_party::hex::{from_hex, to_hex};
use regex::regex;

fn main() -> Result<()> {
    let mut stream = TcpStream::connect(("based.challs.olicyber.it", 10600))?;
    stream.set_nodelay(true)?;
    println!("[+] Connesso al server!");

    // Regex per intercettare l'operazione ovunque si trovi nel flusso di byte
    let message = regex!(r#"\{"message": "([a-zA-Z0-9_=]+)"\}"#);

    let mut raw_buffer = [0; 8192];
    let mut accumulator = String::new();
    let mut count = 0;

    loop {
        // Legge immediatamente i byte disponibili sul socket senza aspettare il '\n'
        let bytes_read = stream.read(&mut raw_buffer)?;
        if bytes_read == 0 {
            println!("[-] Connessione chiusa dal server al ciclo {}.", count);
            break;
        }

        // Accumula i dati ricevuti
        accumulator.push_str(&String::from_utf8_lossy(&raw_buffer[..bytes_read]));
        println!("{}", accumulator);

        // Controlla se nell'accumulatore è comparsa un'operazione valida
        if let Some(captures) = message.captures(&accumulator) {
            count += 1;

            let full_match = captures.get(0).unwrap();
            let message = captures.get(1).unwrap().as_str();
            let result = if accumulator.contains("da binario") {
                from_bin(&("0".to_string() + message)).unwrap()
            } else if accumulator.contains("a binario") {
                to_bin(message)[1..].to_string()
            } else if accumulator.contains("da esadecimale") {
                from_hex(message).unwrap()
            } else if accumulator.contains("a esadecimale") {
                to_hex(message)
            } else if accumulator.contains("da base64") {
                from_b64(message).unwrap()
            } else if accumulator.contains("a base64") {
                to_b64(message)
            } else {
                panic!("{}", accumulator)
            };

            // Invia la risposta al server e svuota il buffer di rete immediatamente
            writeln!(stream, r#"{{"answer":"{}"}}"#, result)?;
            stream.flush()?;

            println!("[{}] Elaborazione in corso...", count);

            // Taglia via dall'accumulatore solo la parte di testo che abbiamo già calcolato
            let match_end = full_match.end();
            accumulator = accumulator[match_end..].to_string();
        }
    }

    Ok(())
}
