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


use std::collections::BTreeMap;
use std::error::Error;

use xmlrpc::Transport;
use xmlrpc::Request as XMLRequest;

use reqwest::header::Cookie;
use reqwest::Response;

use request::Request;

use super::RequestError;
use super::Value;


pub(crate) struct Connection {
    pub(crate) testing: bool,
    pub(crate) debug: bool,
    pub(crate) cookies: Cookie,
}


impl Connection {
    pub(crate) fn new(testing: bool, debug: bool) -> Self {
        Self {
            testing,
            debug,
            cookies: Cookie::new(),
        }
    }
    pub(crate) fn url(&self) -> &'static str {
        const TESTING_URL: &'static str = "https://api.ote.domrobot.com/xmlrpc/";
        const URL: &'static str = "https://api.domrobot.com/xmlrpc/";

        match self.testing {
            true => TESTING_URL,
            false => URL,
        }
    }

    pub(crate) fn send(
        &mut self,
        req: &Request,
    ) -> Result<Option<BTreeMap<String, Value>>, RequestError> {
        let res = {
            let tp = INWXTransport { conn: self };
            let res = req.build().call(tp);
            res.map_err(|_| RequestError::SendFailed)?
        };

        const E: RequestError = RequestError::InvalidResponse;

        if self.debug {
            println!("Response: {:?}", res);
        }

        let res = res.as_struct().ok_or(E)?;
        let code = res.get("code").ok_or(E)?.as_i32().ok_or(E)?;
        let msg = res.get("msg").ok_or(E)?.as_str().ok_or(E)?;

        match code {
            1000 | 1500 => {
                if let Some(data) = res.get("resData") {
                    Ok(Some(data.as_struct().ok_or(E)?.clone()))
                } else {
                    Ok(None)
                }
            }
            _ => Err(RequestError::CallError(code, msg.to_string())),
        }
    }
}


struct INWXTransport<'a> {
    conn: &'a mut Connection,
}


impl<'a> Transport for INWXTransport<'a> {
    type Stream = Response;

    fn transmit(self, request: &XMLRequest) -> Result<Self::Stream, Box<Error + Send + Sync>> {
        use xmlrpc::http::{build_headers, check_response};
        use reqwest::Client;
        use reqwest::header::SetCookie;
        use std::mem::replace;

        let mut body = Vec::new();
        request.write_as_xml(&mut body).unwrap();

        let mut req = Client::new().post(self.conn.url());
        build_headers(&mut req, body.len() as u64);

        req.header(self.conn.cookies.clone());

        req.body(body);
        let res = req.send()?;
        check_response(&res)?;

        // eval the domrobot cookie
        if let Some(&SetCookie(ref content)) = res.headers().get() {
            let mut cookies = Cookie::new();
            for cookie in content {
                let mut parts = cookie.split(";").nth(0).unwrap().split("=");
                let key = parts.nth(0);
                let value = parts.nth(0);
                match (key, value) {
                    (Some(ref key), Some(ref value)) => {
                        cookies.set(key.to_string(), value.to_string());
                    }
                    _ => {
                        // ignore invalid cookies
                    }
                }
            }

            replace(&mut self.conn.cookies, cookies);
        }

        Ok(res)
    }
}
