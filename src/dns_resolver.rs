extern crate trust_dns;
extern crate tokio;
extern crate futures;

use std::str::FromStr;
use tokio::runtime::current_thread::Runtime;

use trust_dns::udp::UdpClientStream;
use trust_dns::client::{ClientFuture, ClientHandle};
use trust_dns::rr::{DNSClass, Name, RData, RecordType};

use futures::future::join_all;


pub fn resolve_domain(domains: &Vec<&str>, result: &mut Vec<String>) {
// We'll be using the current threads Tokio Runtime
let mut runtime = Runtime::new().unwrap();

// We need a connection, TCP and UDP are supported by DNS servers
//   (tcp construction is slightly different as it needs a multiplexer)
let stream = UdpClientStream::new(([8,8,8,8], 53).into());

// Create a new client, the bg is a background future which handles
//   the multiplexing of the DNS requests to the server.
//   the client is a handle to an unbounded queue for sending requests via the
//   background. The background must be scheduled to run before the client can
//   send any dns requests
let (bg, mut client) = ClientFuture::connect(stream);

// run the background task
runtime.spawn(bg);

let mut all_futures = Vec::with_capacity(domains.len());

for domain in domains {
  // Create a query future
  all_futures.push(client.query(Name::from_str(domain).unwrap(), DNSClass::IN, RecordType::A));
}

// turn all futures into one big one
let mega_future = join_all(all_futures);

let responses = runtime.block_on(mega_future).unwrap();

for response in responses {
  for a in response.answers() {
    if let &RData::CNAME(name) = &a.rdata() {
      result.push(name.to_string());
    }
  }
}

// validate it's what we expected
// if let &RData::A(addr) = response.answers()[0].rdata() {
//     assert_eq!(addr, Ipv4Addr::new(93, 184, 216, 34));
// }
}

#[test]
fn test_resolver() {
    let mut result: Vec<String> = Vec::new();
    resolve_domain(&vec!["www.bax-shop.nl", "www.googleadservices.com"], &mut result);
    for s in result {
        let d = &s[.. s.len() - 1];
        println!("CNAME: {}", d);
    }
}