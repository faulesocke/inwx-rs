// Copyright 2018 Urs Schulz
//
// This file is part of inwx-rs.
//
// inwx-rs is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// inwx-rs is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with inwx-rs.  If not, see <http://www.gnu.org/licenses/>.

/// This is a small program I run as a service on my home server to set my dyndns IP address.
/// It uses `igd` to retrieve the current router IP. Then it uses `inwx` to update a dns record.
/// It also reads it's configuration from a `.toml` file.
/// This configuration file is in the form:
///
/// ```norust
/// [inwx]
/// user = "foo"
/// pass = "bar"
/// domain = "foo.bar"
/// record = "home.foo.bar"
///
/// [gateway]
/// search_iface = "192.168.1.1"
/// ```

extern crate inwx;
extern crate igd;
extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::path::Path;
use std::net::Ipv4Addr;


#[derive(Deserialize)]
struct INWXConfig {
    user: String,
    pass: String,
    domain: String,
    record: String,
}

#[derive(Deserialize)]
struct GatewayConfig {
    search_iface: Option<Ipv4Addr>,
}

#[derive(Deserialize)]
struct Config {
    inwx: INWXConfig,
    gateway: Option<GatewayConfig>,
}


fn read_config(path: &Path) -> Config {
    use std::fs::File;
    use std::io::Read;

    let mut data = String::new();

    let mut f = File::open(path).expect(
        format!("Config file {} not found", path.to_str().unwrap())
            .as_ref(),
    );

    f.read_to_string(&mut data).expect(
        "Failed to read config file",
    );

    toml::from_str(&data).expect("Failed to parse config file")
}


fn get_public_ip(cfg: &Config) -> Ipv4Addr {
    let gateway = match cfg.gateway {
        Some(GatewayConfig { search_iface: Some(iface) }) => igd::search_gateway_from(iface),
        _ => igd::search_gateway(),
    }.expect("No gateway found");

    gateway.get_external_ip().expect("Public IP not found")
}


fn main() -> Result<(), usize> {
    let cfg = {
        let mut args: Vec<_> = std::env::args().collect();
        if args.len() != 2 {
            println!("Usage: {} <config_file>", args[0]);
            return Err(1);
        }

        args.remove(1)
    };

    // read config
    let cfg = read_config(Path::new(&cfg));

    // get router public ip
    let ip = get_public_ip(&cfg);
    println!("Public IP is: {}", ip);

    // inwx login
    let mut dr = inwx::Domrobot::new(false, false);
    dr.account.login(&cfg.inwx.user, &cfg.inwx.pass).expect(
        "INWX login failed",
    );

    let info = dr.nameserver.info(&cfg.inwx.domain).expect(
        "Failed to retrieve domain info",
    );

    println!("Current record set:");
    let mut v4rec = None;
    for record in &info.records {
        println!("    {:?}", record);

        if record.typ == "A" && record.name == cfg.inwx.record {
            v4rec = Some(record.clone());
        }
    }

    let mut v4rec = v4rec.expect("Record not found or type is not A");
    println!("Record is {:?}", v4rec);

    v4rec.content = ip.to_string();
    println!("Record after update {:?}", v4rec);

    dr.nameserver.update_record(&v4rec).expect(
        "Failed to update record",
    );

    println!("Records after update:");
    for record in &info.records {
        println!("    {:?}", record);
    }

    dr.account.logout().expect("Failed to logout");

    Ok(())
}
