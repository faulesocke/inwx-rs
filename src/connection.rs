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


use std::collections::{BTreeMap, HashMap};
use std::error::Error;

use xmlrpc::Request as XMLRequest;
use xmlrpc::Transport;

use request::Request;
use reqwest::header::HeaderMap;
use reqwest::Response;

use super::RequestError;
use super::Value;


pub(crate) struct Connection {
    pub(crate) testing: bool,
    pub(crate) debug: bool,
    pub(crate) cookies: HashMap<String, String>,
}


impl Connection {
    pub(crate) fn new(testing: bool, debug: bool) -> Self {
        Self {
            testing,
            debug,
            cookies: HashMap::new(),
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

    /// Sends a request and returns the response data as a map
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

    fn header_map(&self) -> HeaderMap {
        use reqwest::header::COOKIE;

        self.cookies
            .iter()
            .map(|(n, v)| (COOKIE, format!("{}={}", n, v).parse().unwrap()))
            .collect()
    }
}


struct INWXTransport<'a> {
    conn: &'a mut Connection,
}


impl<'a> Transport for INWXTransport<'a> {
    type Stream = Response;

    fn transmit(self, request: &XMLRequest) -> Result<Self::Stream, Box<Error + Send + Sync>> {
        use reqwest::Client;
        use xmlrpc::http::{build_headers, check_response};

        let mut body = Vec::new();
        request.write_as_xml(&mut body).unwrap();

        let req = Client::new().post(self.conn.url());
        let req = build_headers(req, body.len() as u64);

        let res = req.headers(self.conn.header_map()).body(body).send()?;
        check_response(&res)?;

        // update connection cookies
        for cookie in res.cookies() {
            self.conn
                .cookies
                .insert(cookie.name().into(), cookie.value().into());
        }

        Ok(res)
    }
}
