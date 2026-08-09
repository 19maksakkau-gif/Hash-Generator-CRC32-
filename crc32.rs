// crc32.rs
use std::fs::File;
use std::io::{self, Read, BufReader};
use std::path::Path;
use std::env;
use clap::{Arg, Command};
use crc::{Crc, Algorithm};

const CRC32_IEEE: Algorithm<u32> = Algorithm {
    width: 32,
    poly: 0x04C11DB7,
    init: 0xFFFFFFFF,
    refin: true,
    refout: true,
    xorout: 0xFFFFFFFF,
    check: 0xCBF43926,
    residue: 0xDEBB20E3,
};

const CRC32C: Algorithm<u32> = Algorithm {
    width: 32,
    poly: 0x1EDC6F41,
    init: 0xFFFFFFFF,
    refin: true,
    refout: true,
    xorout: 0xFFFFFFFF,
    check: 0xE3069283,
    residue: 0x48674BC7,
};

fn compute_crc32(data: &[u8], use_crc32c: bool) -> u32 {
    if use_crc32c {
        let crc = Crc::<u32>::new(&CRC32C);
        let mut digest = crc.digest();
        digest.update(data);
        digest.finalize()
    } else {
        let crc = Crc::<u32>::new(&CRC32_IEEE);
        let mut digest = crc.digest();
        digest.update(data);
        digest.finalize()
    }
}

fn compute_file_crc32(filename: &str, use_crc32c: bool) -> Result<u32, io::Error> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 8192];
    if use_crc32c {
        let crc = Crc::<u32>::new(&CRC32C);
        let mut digest = crc.digest();
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 { break; }
            digest.update(&buffer[..n]);
        }
        Ok(digest.finalize())
    } else {
        let crc = Crc::<u32>::new(&CRC32_IEEE);
        let mut digest = crc.digest();
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 { break; }
            digest.update(&buffer[..n]);
        }
        Ok(digest.finalize())
    }
}

fn format_hash(val: u32, format: &str) -> String {
    match format {
        "hex" => format!("0x{:08X}", val),
        "dec" => format!("{}", val),
        "bin" => format!("{:032b}", val),
        _ => format!("0x{:08X}", val),
    }
}

fn main() {
    let matches = Command::new("crc32")
        .version("1.0")
        .about("CRC32 Hash Generator")
        .arg(Arg::new("inputs").multiple_occurrences(true).help("Строки или файлы"))
        .arg(Arg::new("crc32c").long("crc32c").help("Использовать CRC32C"))
        .arg(Arg::new("check").long("check").takes_value(true).help("Сравнить с хэшем (HEX)"))
        .arg(Arg::new("dec").long("dec").help("Вывод в десятичном формате"))
        .arg(Arg::new("bin").long("bin").help("Вывод в бинарном формате"))
        .arg(Arg::new("progress").long("progress").help("Показывать прогресс"))
        .get_matches();

    let use_crc32c = matches.is_present("crc32c");
    let check = matches.value_of("check").map(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16));
    let format = if matches.is_present("dec") { "dec" } else if matches.is_present("bin") { "bin" } else { "hex" };
    let inputs: Vec<&str> = matches.get_many("inputs").unwrap_or_default().map(|s| *s).collect();

    if inputs.is_empty() {
        // Проверяем stdin
        let mut buffer = Vec::new();
        if let Ok(n) = io::stdin().read_to_end(&mut buffer) {
            if n > 0 {
                let hash_val = compute_crc32(&buffer, use_crc32c);
                println!("{}", format_hash(hash_val, format));
                return;
            }
        }
        eprintln!("Не указаны данные. Используйте: crc32 <строка или файл>");
        std::process::exit(1);
    }

    for item in inputs {
        let path = Path::new(item);
        if path.exists() && !path.is_dir() {
            match compute_file_crc32(item, use_crc32c) {
                Ok(hash_val) => {
                    let mut output = format!("{}: {}", item, format_hash(hash_val, format));
                    if let Some(Ok(expected)) = check {
                        let status = if expected == hash_val { "✅ OK" } else { "❌ FAIL" };
                        output.push_str(&format!(" (check: {})", status));
                    }
                    println!("{}", output);
                }
                Err(e) => eprintln!("Ошибка чтения {}: {}", item, e),
            }
        } else {
            let data = item.as_bytes();
            let hash_val = compute_crc32(data, use_crc32c);
            let mut output = format_hash(hash_val, format);
            if let Some(Ok(expected)) = check {
                let status = if expected == hash_val { "✅ OK" } else { "❌ FAIL" };
                output.push_str(&format!(" (check: {})", status));
            }
            println!("{}", output);
        }
    }
}
