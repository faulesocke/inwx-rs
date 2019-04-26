// Copyright 2018-2019 Urs Schulz
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

#![deny(warnings)]

extern crate reqwest;
extern crate xmlrpc;

pub mod account;
mod connection;
pub mod nameserver;
pub mod request;


use std::sync::Arc;
use std::sync::Mutex;

use xmlrpc::Value;

use crate::account::Account;
use crate::connection::Connection;
use crate::nameserver::Nameserver;


pub struct Domrobot {
    pub account: Account,
    pub nameserver: Nameserver,
}


#[derive(Debug, PartialEq)]
pub enum RequestError {
    NotLoggedIn,
    LoginFailed,
    SendFailed,
    InvalidResponse,
    CallError(i32, String),
}


impl Domrobot {
    pub fn new(testing: bool, debug: bool) -> Self {
        let conn = Arc::new(Mutex::new(Connection::new(testing, debug)));

        Self {
            account: Account { conn: conn.clone() },
            nameserver: Nameserver { conn: conn.clone() },
        }
    }
}
