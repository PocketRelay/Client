use crate::{api::TARGET, constants::TELEMETRY_PORT, ui::show_error};
use reqwest::Client;
use serde::Serialize;
use std::{io, net::Ipv4Addr, process::exit};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

/// Server API endpoint to send telemetry data to
const TELEMETRY_ENDPOINT: &str = "api/server/telemetry";

pub async fn start_server(http_client: Client) {
    // Initializing the underlying TCP listener
    let listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, TELEMETRY_PORT)).await {
        Ok(value) => value,
        Err(err) => {
            let text = format!("Failed to start telemetry: {}", err);
            show_error("Failed to start", &text);
            exit(1);
        }
    };

    // Accept incoming connections
    loop {
        let stream: TcpStream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(_) => continue,
        };

        let http_client = http_client.clone();

        tokio::spawn(async move {
            let target = match &*TARGET.read().await {
                Some(value) => value.clone(),
                None => return,
            };

            let url = target
                .url
                .join(TELEMETRY_ENDPOINT)
                .expect("Failed to create telemetry endpoint");

            let mut stream = stream;
            while let Ok(message) = read_message(&mut stream).await {
                // TODO: Batch these telemetry messages and send them to the server
                let _ = http_client.post(url.clone()).json(&message).send().await;
            }
        });
    }
}

/// Reads a telemetry message buffer from the provided input
/// stream returning the buffer that was read.
///
/// `stream` The stream to read from
async fn read_message(stream: &mut TcpStream) -> io::Result<TelemetryMessage> {
    let length = {
        // Buffer for reading the header + padding + legnth bytes
        let mut header = [0u8; 12];
        stream.read_exact(&mut header).await?;
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&header[10..]);
        u16::from_be_bytes(bytes)
    };

    // Remove the header size from the message length
    let length = (length - 12.min(length)) as usize;

    // Create a new buffer of the expected size
    let mut buffer = vec![0u8; length];
    stream.read_exact(&mut buffer).await?;

    let message = decode_message(buffer);
    Ok(message)
}

// Structure containing key value pairs for telemetry messages
#[derive(Debug, Serialize)]
pub struct TelemetryMessage {
    // Vec of key values
    pub values: Vec<(String, String)>,
}

/// TLM3 key for decoding the TML3 line
const TLM3_KEY: &[u8] = b"The truth is back in style.";

/// Decodes the telemetry message from the message buffer into
/// a telemetry message structure
///
/// `message` The raw message bytes
fn decode_message(message: Vec<u8>) -> TelemetryMessage {
    // Split the buffer into pairs of values
    let values: Vec<(String, String)> = message
        // Split the message into new lines
        .split(|value| b'\n'.eq(value))
        // Filter only on the key=value pair lines
        .filter_map(|slice| {
            let mut parts = slice.splitn(2, |value| b'='.eq(value));
            let first = parts.next()?;
            let second = parts.next()?;
            Some((first, second))
        })
        // Handle decoding the values
        .map(|(key, value)| {
            let key = String::from_utf8_lossy(key).to_string();
            let value = if key.eq("TLM3") {
                tlm3(value)
            } else {
                format!("{:?}", value)
            };

            (key, value)
        })
        .collect();

    TelemetryMessage { values }
}

fn tlm3(input: &[u8]) -> String {
    input
        .splitn(2, |value| b'-'.eq(value))
        .nth(1)
        .map(|line| {
            let value = xor_cipher(line, TLM3_KEY);
            // Safety: Characters from the xor_cipher are within the valid utf-8 range
            unsafe { String::from_utf8_unchecked(value) }
        })
        .unwrap_or_else(|| format!("{:?}", input))
}

fn xor_cipher(input: &[u8], key: &[u8]) -> Vec<u8> {
    input
        .iter()
        // Copy the data bytes
        .copied()
        // Iterate along-side the key
        .zip(key.iter().cycle().copied())
        // Process the next value using the key
        .map(|(data, key)| ((data ^ key) % 0x80))
        // Collect the processed bytes
        .collect()
}

#[cfg(test)]
mod test {
    use crate::servers::telemetry::tlm3;

    use super::{xor_cipher, TLM3_KEY};

    #[test]
    fn test_xor_cipher() {
        // Data that should be decodable
        let test_data =
            "123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

        let enc_data = xor_cipher(test_data.as_bytes(), TLM3_KEY);
        let dec_data = xor_cipher(&enc_data, TLM3_KEY);

        assert_eq!(&dec_data, test_data.as_bytes());
    }

    #[test]
    fn test_known_data() {
        let enc_data = &[
            100, 88, 85, 144, 68, 64, 49, 50, 71, 141, 82, 67, 144, 82, 81, 83, 91, 146, 91, 65,
            98, 60, 59, 45, 67, 54, 107, 135, 59, 74, 111, 56, 60, 50, 91, 30, 76, 135, 148, 29,
            43, 47, 55, 77, 84, 133, 128, 71, 78, 189, 55, 56, 73, 30, 100, 88, 85, 144, 70, 54,
            51, 91, 69, 27, 89, 67, 144, 82, 81, 83, 89, 147, 70, 33, 110, 63, 58, 86, 46, 41, 111,
            142, 71, 33, 99, 59, 60, 90, 22, 141, 82, 27, 78, 141, 80, 80, 87, 93, 22, 90, 95, 22,
            75, 68, 95, 138, 22, 90, 53, 85, 84, 145, 82, 134, 134, 128, 137, 29, 90, 85, 83, 135,
            146, 144, 86, 80, 138, 25, 68, 25, 128, 54, 47, 51, 94, 144, 104,
        ];
        let expected = "000002DF/-;00000022/BOOT/SESS/OLNG/vlng=INT&tlng=INT,000002DF/-;00000023/ONLN/BLAZ/DCON/berr=-2146631680&fsta=11&tsta=3&sess=pcwdjtOCVpD\0";
        let dec_data = xor_cipher(enc_data, TLM3_KEY);

        assert_eq!(&dec_data, expected.as_bytes());
    }

    #[test]
    fn test_tlm3_line() {
        let enc_data = &mut [
            64, 56, 97, 45, 100, 88, 85, 144, 68, 64, 49, 50, 71, 141, 82, 67, 144, 82, 81, 83, 91,
            146, 91, 65, 98, 60, 59, 45, 67, 54, 107, 135, 59, 74, 111, 56, 60, 50, 91, 30, 76,
            135, 148, 29, 43, 47, 55, 77, 84, 133, 128, 71, 78, 189, 55, 56, 73, 30, 100, 88, 85,
            144, 70, 54, 51, 91, 69, 27, 89, 67, 144, 82, 81, 83, 89, 147, 70, 33, 110, 63, 58, 86,
            46, 41, 111, 142, 71, 33, 99, 59, 60, 90, 22, 141, 82, 27, 78, 141, 80, 80, 87, 93, 22,
            90, 95, 22, 75, 68, 95, 138, 22, 90, 53, 85, 84, 145, 82, 134, 134, 128, 137, 29, 90,
            85, 83, 135, 146, 144, 86, 80, 138, 25, 68, 25, 128, 54, 47, 51, 94, 144, 104,
        ];
        let expected = "000002DF/-;00000022/BOOT/SESS/OLNG/vlng=INT&tlng=INT,000002DF/-;00000023/ONLN/BLAZ/DCON/berr=-2146631680&fsta=11&tsta=3&sess=pcwdjtOCVpD\0";
        let dec_data = tlm3(enc_data);
        assert_eq!(dec_data.as_bytes(), expected.as_bytes());
    }
}
