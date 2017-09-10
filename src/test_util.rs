#![cfg(test)]

use {MEvent, Result};
use futures::Stream;
use futures::stream::iter;
use host::Host;
use std::net::SocketAddr;
use tokio_core::reactor::Core;

pub fn simulate<F, K, E>(setup: F, kill: &K, expectations: E)
where
    F: Fn(&Host, &Host) -> Result<()>,
    K: Fn(&MEvent) -> bool,
    E: Fn(SocketAddr, SocketAddr) -> Vec<MEvent>,
{
    let (h1, stream1) = Host::new("127.0.0.1:0").expect("host 1");
    let (h2, stream2) = Host::new("127.0.0.1:0").expect("host 2");

    let h1addr = h1.addr;
    let h2addr = h2.addr;

    let ok = |v| -> Result<MEvent> { Ok(v) };
    let killswitch = |host: Host| {
        move |event| {
            if kill(&event) {
                host.shutdown().expect("host shutdown");
            }
            event
        }
    };

    setup(&h1, &h2).expect("test frame setup");

    let mut core = Core::new().expect("built core");
    let h1_kill = &killswitch(h1);
    let h2_kill = &killswitch(h2);

    assert_eq!(
        core.run(
            iter(stream1.map(h1_kill).map(&ok))
                .select(iter(stream2.map(h2_kill).map(&ok)))
                .collect(),
        ).expect("user events"),
        expectations(h1addr, h2addr)
    );
}
