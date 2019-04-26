# inwx

[![crates.io](https://img.shields.io/crates/v/inwx.svg)](https://crates.io/crates/inwx)
[![docs.rs](https://docs.rs/inwx/badge.svg)](https://docs.rs/inwx/)

This is an unofficial Rust binding for the inwx domrobot API. Currently it only has the following features:

 * Login and retrieve account information (`account.login` via method `inwx.account.login`)
 * Logout (`account.logout` via method `inwx.account.logout`)
 * Retrieve information and all records for a specific domain (`nameserver.info` via method `inwx.nameserver.info`)
 * Update records (`nameserver.updateRecord` via method `inwx.nameserver.update_record`)

**If you need a certain function implemented, just ask in an issue and I will probably add it.**


## Usage
Add the following to your `Cargo.toml`:

```toml
[dependencies]
inwx = "0.1.0"
```

See the example `simple_query` for information about connecting to the domrobot API and retrieving all records for a given domain. See the `dyndns` example for some more complex operations.


## Examples
Currently there are two examples: `simple_query` retrieves all records for a specific domain and lists them. `dyndns` is more interesting, it loads a configuration file which contains inwx credentials, then retrieves the current IP address of the network gateway via igd and then updates a specified DNS record to point to that address.


## Using the client for dynamic DNS
The `dyndns` example is a fully-featured dyndns client which retrieves your current routers public IP address and updates a record on your inwx-hosted domain. To use it, you first have to write a little configuration file:

```toml
[inwx]
user = "foo"
pass = "bar"
domain = "foo.bar"
record = "home.foo.bar"

[gateway]
search_iface = "192.168.1.1"
```

Set `search_iface` to the IP address of your network interface. The program will search a router on this interface using the _Internet Gateway Protocol_, retrieve it's public IP address and then update the specified record of the specified domain to that IP address (it will only update `A` records).

To run it, save your configuration as `foobar.toml` and then invoke:
```
cargo run --example=dyndns --release -- foobar.toml
```


## License
This project is licensed under the GNU AGPL version 3 or later, see the `LICENSE` file for more information.
