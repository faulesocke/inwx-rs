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


extern crate xmlrpc;
extern crate reqwest;

use xmlrpc::Request as XMLRequest;
use xmlrpc::Transport;
pub use xmlrpc::Value;

use std::collections::BTreeMap;
use reqwest::header::Cookie;

use std::error::Error;


pub struct Request {
    method: String,
    params: BTreeMap<String, Value>,
}


impl Request {
    pub fn new(method: &str) -> Self {
        Self {
            method: method.to_owned(),
            params: BTreeMap::new(),
        }
    }

    pub fn param(&mut self, name: &str, value: Value) -> Option<Value> {
        self.params.insert(name.to_owned(), value)
    }

    fn build(&self) -> XMLRequest {
        XMLRequest::new(&self.method).arg(Value::Struct(self.params.clone()))
    }
}


#[derive(Debug)]
pub struct LoginInfo {
    pub customer_id: i32,
    pub account_id: i32,
    pub tfa: String,
    pub builddate: String,
    pub version: String,
}


#[derive(Debug)]
pub struct DomainInfo {
    pub ro_id: i32,
    pub domain: String,
    pub typ: String,
    pub slave_dns: Vec<(String, String)>,
    pub records: Vec<DomainRecord>,
}


#[derive(Debug)]
pub struct DomainRecord {
    pub id: i32,
    pub name: String,
    pub typ: String,
    pub content: String,
    pub ttl: i32,
    pub prio: i32,
}


pub struct Domrobot {
    testing: bool,
    cookies: Cookie,
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
    pub fn new(testing: bool) -> Self {
        Self {
            testing: testing,
            cookies: Cookie::new(),
        }
    }
    pub fn url(&self) -> &'static str {
        const TESTING_URL: &'static str = "https://api.ote.domrobot.com/xmlrpc/";
        const URL: &'static str = "https://api.domrobot.com/xmlrpc/";

        match self.testing {
            true => TESTING_URL,
            false => URL,
        }
    }

    pub fn send(&mut self, req: &Request) -> Result<BTreeMap<String, Value>, RequestError> {
        let tp = INWXTransport { robot: self };
        let res = req.build().call(tp);
        let res = res.map_err(|_| RequestError::SendFailed)?;

        const E: RequestError = RequestError::InvalidResponse;

        let res = res.as_struct().ok_or(E)?;
        let code = res.get("code").ok_or(E)?.as_i32().ok_or(E)?;
        let msg = res.get("msg").ok_or(E)?.as_str().ok_or(E)?;

        match code {
            1000 => {
                let data = res.get("resData").ok_or(E)?.as_struct().ok_or(E)?;
                Ok(data.clone())
            }
            1500 => Ok(BTreeMap::new()),
            _ => Err(RequestError::CallError(code, msg.to_string())),
        }
    }

    pub fn account_login(&mut self, user: &str, pass: &str) -> Result<LoginInfo, RequestError> {
        let mut req = Request::new("account.login");
        req.param("user", Value::from(user));
        req.param("pass", Value::from(pass));
        let res = self.send(&req)?;

        // first check, that we now have a domrobot cookie
        if self.cookies.get("domrobot").is_none() {
            return Err(RequestError::LoginFailed);
        }

        // parse info
        const E: RequestError = RequestError::InvalidResponse;
        Ok(LoginInfo {
            customer_id: res.get("customerId").ok_or(E)?.as_i32().ok_or(E)?,
            account_id: res.get("accountId").ok_or(E)?.as_i32().ok_or(E)?,
            tfa: res.get("tfa").ok_or(E)?.as_str().ok_or(E)?.to_owned(),
            builddate: res.get("builddate").ok_or(E)?.as_str().ok_or(E)?.to_owned(),
            version: res.get("version").ok_or(E)?.as_str().ok_or(E)?.to_owned(),
        })
    }

    pub fn account_logout(&mut self) -> Result<(), RequestError> {
        self.send(&Request::new("account.logout"))?;
        Ok(())
    }

    pub fn nameserver_info(&mut self, domain: &str) -> Result<DomainInfo, RequestError> {
        const E: RequestError = RequestError::InvalidResponse;

        let mut req = Request::new("nameserver.info");
        req.param("domain", Value::from(domain));
        let res = self.send(&req)?;

        let domain = res.get("domain").ok_or(E)?.as_str().ok_or(E)?.to_owned();
        let ro_id = res.get("roId").ok_or(E)?.as_i32().ok_or(E)?;
        let typ = res.get("type").ok_or(E)?.as_str().ok_or(E)?.to_owned();

        let mut slave_dns = Vec::new();
        if let Some(arr) = res.get("slaveDns") {
            for entry in arr.as_array().ok_or(E)? {
                let entry = entry.as_struct().ok_or(E)?;
                let name = entry.get("name").ok_or(E)?.as_str().ok_or(E)?.to_owned();
                let ip = entry.get("ip").ok_or(E)?.as_str().ok_or(E)?.to_owned();
                slave_dns.push((name, ip));
            }
        }

        let mut records = Vec::new();
        if let Some(arr) = res.get("record") {
            for record in arr.as_array().ok_or(E)? {
                let record = record.as_struct().ok_or(E)?;
                let id = record.get("id").ok_or(E)?.as_i32().ok_or(E)?;
                let name = record.get("name").ok_or(E)?.as_str().ok_or(E)?.to_owned();
                let typ = record.get("type").ok_or(E)?.as_str().ok_or(E)?.to_owned();
                let content = record
                    .get("content")
                    .ok_or(E)?
                    .as_str()
                    .ok_or(E)?
                    .to_owned();
                let ttl = record.get("ttl").ok_or(E)?.as_i32().ok_or(E)?;
                let prio = record.get("prio").ok_or(E)?.as_i32().ok_or(E)?;
                records.push(DomainRecord {
                    id,
                    name,
                    typ,
                    content,
                    ttl,
                    prio,
                });
            }
        }

        Ok(DomainInfo {
            domain,
            ro_id,
            typ,
            slave_dns,
            records,
        })
    }

    pub fn nameserver_update_record(&mut self, record: &DomainRecord) -> Result<(), RequestError> {
        let mut req = Request::new("nameserver.updateRecord");

        req.param("id", Value::from(record.id));
        req.param("name", Value::from(&*record.name));
        req.param("type", Value::from(&*record.typ));
        req.param("content", Value::from(&*record.content));
        req.param("prio", Value::from(record.prio));
        req.param("ttl", Value::from(record.ttl));

        self.send(&req)?;
        Ok(())
    }
}


struct INWXTransport<'a> {
    robot: &'a mut Domrobot,
}


impl<'a> Transport for INWXTransport<'a> {
    type Stream = reqwest::Response;

    fn transmit(self, request: &XMLRequest) -> Result<Self::Stream, Box<Error + Send + Sync>> {
        use xmlrpc::http::{build_headers, check_response};
        use reqwest::Client;
        use reqwest::header::SetCookie;

        let mut body = Vec::new();
        request.write_as_xml(&mut body).unwrap();

        let mut req = Client::new().post(self.robot.url());
        build_headers(&mut req, body.len() as u64);

        req.header(self.robot.cookies.clone());

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

            std::mem::replace(&mut self.robot.cookies, cookies);
        }

        Ok(res)
    }
}
