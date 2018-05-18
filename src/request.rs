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

use xmlrpc::Request as XMLRequest;
pub use xmlrpc::Value;


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

    pub(crate) fn build(&self) -> XMLRequest {
        XMLRequest::new(&self.method).arg(Value::Struct(self.params.clone()))
    }
}
