#![cfg(test)]

use {MEvent, Result};
use futures::Stream;
use futures::stream::iter;
use node::Node;
use std::net::SocketAddr;
use tokio_core::reactor::Core;

pub fn simulate<F, K, E>(setup: F, kill: &K, expectations: E)
where
    F: Fn(&Node, &Node) -> Result<()>,
    K: Fn(&MEvent) -> bool,
    E: Fn(SocketAddr, SocketAddr) -> Vec<MEvent>,
{
    let (n1, stream1) = Node::new("127.0.0.1:0").expect("node 1");
    let (n2, stream2) = Node::new("127.0.0.1:0").expect("node 2");

    let n1addr = n1.addr;
    let n2addr = n2.addr;

    let ok = |v| -> Result<MEvent> { Ok(v) };
    let killswitch = |node: Node| {
        move |event| {
            if kill(&event) {
                node.shutdown().expect("node shutdown");
            }
            event
        }
    };

    setup(&n1, &n2).expect("test frame setup");

    let mut core = Core::new().expect("built core");
    let n1_kill = &killswitch(n1);
    let n2_kill = &killswitch(n2);

    assert_eq!(
        core.run(
            iter(stream1.map(n1_kill).map(&ok))
                .select(iter(stream2.map(n2_kill).map(&ok)))
                .collect(),
        ).expect("user events"),
        expectations(n1addr, n2addr)
    );
}
