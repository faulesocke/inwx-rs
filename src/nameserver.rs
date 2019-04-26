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

use crate::request::Request;

use crate::connection::Connection;

use super::RequestError;
use super::Value;


#[derive(Debug)]
pub struct DomainInfo {
    pub ro_id: i32,
    pub domain: String,
    pub typ: String,
    pub slave_dns: Vec<(String, String)>,
    pub records: Vec<DomainRecord>,
}


#[derive(Debug, Clone)]
pub struct DomainRecord {
    pub id: i32,
    pub name: String,
    pub typ: String,
    pub content: String,
    pub ttl: i32,
    pub prio: i32,
}


pub struct Nameserver {
    pub(crate) conn: Arc<Mutex<Connection>>,
}


impl Nameserver {
    pub fn info(&mut self, domain: &str) -> Result<DomainInfo, RequestError> {
        const E: RequestError = RequestError::InvalidResponse;

        let mut req = Request::new("nameserver.info");
        req.param("domain", Value::from(domain));
        let res = self.conn.lock().unwrap().send(&req)?.ok_or(E)?;

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

    pub fn update_record(&mut self, record: &DomainRecord) -> Result<(), RequestError> {
        let mut req = Request::new("nameserver.updateRecord");

        req.param("id", Value::from(record.id));
        req.param("name", Value::from(&*record.name));
        req.param("type", Value::from(&*record.typ));
        req.param("content", Value::from(&*record.content));
        req.param("prio", Value::from(record.prio));
        req.param("ttl", Value::from(record.ttl));

        self.conn.lock().unwrap().send(&req)?;
        Ok(())
    }
}
