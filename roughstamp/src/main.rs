// Copyright 2017-2021 int08h LLC

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::error;
use std::net::{ToSocketAddrs, UdpSocket};
use std::io::{self, Write};
use std::time::Duration;
use std::path::PathBuf;
use std::fs::File;
use std::convert::TryInto;

use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{Local, TimeZone};
use chrono::offset::Utc;
use clap::{Args, Parser, Subcommand};
use data_encoding::{Encoding, HEXLOWER_PERMISSIVE, BASE64};
use roughenough::{
    RtMessage,
    Tag,
};

const HEX: Encoding = HEXLOWER_PERMISSIVE;

mod stamp;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Args)]
struct GlobalOpts {
    // /// Verbosity level (can be specified multiple times)
    //#[clap(long, short, global = true, parse(from_occurrences))]
    //verbose: usize,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a timestamp
    Stamp {
        /// The digest to timestamp
        digest: String,

        /// Server to timestamp with
        server: String,

        /// Timestamp file
        stamp_file: PathBuf,

        /// Server key
        key: Option<String>,
    },
    /// Verify a timestamp
    Verify {
        /*
        /// An example option
        #[clap(long, short = 'o')]
        example_opt: bool,

        /// The path to read from
        path: Utf8PathBuf,
        // (can #[clap(flatten)] other argument structs here)
        */
    },
}

fn expand_digest(digest: [u8; 32]) -> [u8; 64] {
    let mut r = [0; 64];
    r[0 .. 32].copy_from_slice(&digest);
    for (src, dst) in digest.iter().zip(r[32 .. 64].iter_mut().rev()) {
        *dst = *src;
    }
    r
}

fn hex_digest_to_digest64(digest: &str) -> Result<Vec<u8>, Box<dyn error::Error>> {
    let mut digest32: [u8; 32] = HEX.decode(digest.as_bytes())?.try_into().expect("digest not 32 bytes long");
    let mut digest64 = expand_digest(digest32);
    Ok(digest64.into())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let cli = Cli::parse();

    dbg!(&cli);

    match cli.command {
        Command::Stamp { digest, server, key, stamp_file } => {
            let pub_key = None; // FIXME

            let digest = hex_digest_to_digest64(&digest)?;

            let addr = dbg!(server.to_socket_addrs()?.next().unwrap());
            let req = stamp::make_request(&digest, false);
            let socket = UdpSocket::bind(if addr.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" })?;

            socket.set_read_timeout(Duration::from_secs(1).into())?;

            socket.send_to(&req, addr)?;

            eprintln!("sent");
            let mut buf = [0u8; 4096];
            let resp_len = socket.recv_from(&mut buf)?.0;

            let resp_bytes = &buf[0 .. resp_len];

            verify_timestamp(resp_bytes, pub_key, digest)?;

            let mut fd = File::options().read(true).write(true).create_new(true).open(stamp_file)?;
            fd.write_all(resp_bytes)?;

            Ok(())
        },
        Command::Verify { .. } => {
            todo!()
        },
    }
}

fn verify_timestamp(raw: &[u8], pub_key: Option<Vec<u8>>, digest: Vec<u8>) -> Result<(), Box<dyn error::Error>> {
    let resp = RtMessage::from_bytes(raw).unwrap();

    let stamp::ParsedResponse {
        verified,
        midpoint,
        radius,
    } = dbg!(stamp::ResponseHandler::new(roughenough::version::Version::Classic, pub_key.clone(), resp.clone(), digest)
        .extract_time());

    let map = resp.into_hash_map();
    let index = map[&Tag::INDX]
        .as_slice()
        .read_u32::<LittleEndian>()
        .unwrap();

    let seconds = midpoint / 10_u64.pow(6);
    let nsecs = (midpoint - (seconds * 10_u64.pow(6))) * 10_u64.pow(3);
    let verify_str = if verified { "Yes" } else { "No" };

    /*
    let out = if use_utc {
        let ts = Utc.timestamp(seconds as i64, nsecs as u32);
        ts.format(time_format).to_string()
    } else {
        let ts = Local.timestamp(seconds as i64, nsecs as u32);
        ts.format(time_format).to_string()
    };
    */

    Ok(())
}
