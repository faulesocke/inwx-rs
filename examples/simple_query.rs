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


extern crate inwx;

use inwx::Domrobot;


fn read_line(req: &str) -> String {
    use std::io::stdin;
    use std::io::stdout;
    use std::io::Write;

    let mut stdout = stdout();
    write!(stdout, "{}", req).unwrap();
    stdout.flush().unwrap();

    let stdin = stdin();
    let mut buf = String::new();
    stdin.read_line(&mut buf).unwrap();

    let len = buf.len() - 1;
    buf.truncate(len);
    buf
}


fn main() {
    let name = read_line("Name: ");
    let pass = read_line("Password: ");
    let domain = read_line("Domain: ");

    let mut inwx = Domrobot::new(false);
    let acc_info = inwx.account.login(&name, &pass).unwrap();
    println!("Account info: {:?}", acc_info);

    let info = inwx.nameserver.info(&domain).unwrap();
    println!("Records:");
    for record in info.records {
        println!("{:?}", record);
    }

    inwx.account.logout().unwrap();
}
