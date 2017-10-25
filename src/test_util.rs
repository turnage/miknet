#![cfg(test)]

use {Error, MEvent, Result};
use futures::Stream;
use futures::stream::iter_ok;
use node::Node;
use std::net::SocketAddr;
use tokio_core::reactor::Core;

/// random() will always return this constant in test builds.
pub const RAND_TEST_CONST: u32 = 100;

pub fn random() -> u32 { RAND_TEST_CONST }

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
            iter_ok::<_, Error>(stream1.map(n1_kill))
                .select(iter_ok(stream2.map(n2_kill)))
                .collect(),
        ).expect("user events"),
        expectations(n1addr, n2addr)
    );
}
