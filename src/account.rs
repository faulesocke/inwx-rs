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


use std::sync::Arc;
use std::sync::Mutex;

use super::RequestError;
use super::Value;

use crate::connection::Connection;
use crate::request::Request;


#[derive(Debug)]
pub struct LoginInfo {
    pub customer_id: i32,
    pub account_id: i32,
    pub tfa: String,
    pub builddate: String,
    pub version: String,
}


pub struct Account {
    pub(crate) conn: Arc<Mutex<Connection>>,
}


impl Account {
    pub fn login(&self, user: &str, pass: &str) -> Result<LoginInfo, RequestError> {
        const E: RequestError = RequestError::InvalidResponse;

        let mut req = Request::new("account.login");
        req.param("user", Value::from(user));
        req.param("pass", Value::from(pass));
        let res = self.conn.lock().unwrap().send(&req)?.ok_or(E)?;

        // first check, that we now have a domrobot cookie
        if self.conn.lock().unwrap().cookies.get("domrobot").is_none() {
            return Err(RequestError::LoginFailed);
        }

        // parse info
        Ok(LoginInfo {
            customer_id: res.get("customerId").ok_or(E)?.as_i32().ok_or(E)?,
            account_id: res.get("accountId").ok_or(E)?.as_i32().ok_or(E)?,
            tfa: res.get("tfa").ok_or(E)?.as_str().ok_or(E)?.to_owned(),
            builddate: res.get("builddate").ok_or(E)?.as_str().ok_or(E)?.to_owned(),
            version: res.get("version").ok_or(E)?.as_str().ok_or(E)?.to_owned(),
        })
    }

    pub fn logout(&self) -> Result<(), RequestError> {
        self.conn
            .lock()
            .unwrap()
            .send(&Request::new("account.logout"))?;
        Ok(())
    }
}
