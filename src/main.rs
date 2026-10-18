use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::FromStr;

use anyhow::Result;
use num_bigint::BigInt;
use num_traits::Pow;
use regex::Regex;

fn main() -> Result<()> {
    let mut stream = TcpStream::connect(("2048.challs.olicyber.it", 10007))?;
    stream.set_nodelay(true)?;
    println!("[+] Connesso al server!");

    // Regex per intercettare l'operazione ovunque si trovi nel flusso di byte
    let op_re =
        Regex::new(r"(SOMMA|DIFFERENZA|PRODOTTO|DIVISIONE_INTERA|POTENZA)\s+([0-9]+)\s+([0-9]+)")?;

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

        // Controlla se nell'accumulatore è comparsa un'operazione valida
        if let Some(captures) = op_re.captures(&accumulator) {
            count += 1;

            let full_match = captures.get(0).unwrap();
            let op = captures.get(1).unwrap().as_str();
            let n1 = BigInt::from_str(captures.get(2).unwrap().as_str())?;
            let n2 = BigInt::from_str(captures.get(3).unwrap().as_str())?;

            let result = match op {
                "SOMMA" => n1 + n2,
                "DIFFERENZA" => n1 - n2,
                "PRODOTTO" => n1 * n2,
                "DIVISIONE_INTERA" => n1 / n2,
                "POTENZA" => n1.pow(n2.to_biguint().unwrap()),
                _ => unreachable!(),
            };

            // Invia la risposta al server e svuota il buffer di rete immediatamente
            writeln!(stream, "{}", result)?;
            stream.flush()?;

            // Stampa di monitoraggio leggera (ogni 200 cicli) per non rallentare l'I/O
            if count % 200 == 0 || count > 2040 {
                println!("[{}/2048] Elaborazione in corso...", count);
            }

            if count == 2048 {
                println!("[+] 2048 operazioni completate! Ricezione flag in corso...");
                // Svuota l'accumulatore per leggere la flag finale
                accumulator.clear();
                stream.read_to_string(&mut accumulator)?;
                println!("\n[+] FLAG:\n{}", accumulator);
                break;
            }

            // Taglia via dall'accumulatore solo la parte di testo che abbiamo già calcolato
            let match_end = full_match.end();
            accumulator = accumulator[match_end..].to_string();
        }
    }

    Ok(())
}
