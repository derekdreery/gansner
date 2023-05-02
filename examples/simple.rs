//! A simple example of library use.

use gansner::Gansner;
use kurbo::Size;

fn main() {
    const SZ: Size = Size::new(10., 10.);
    let mut g = Gansner::new();
    let a = g.add_node((), SZ);
    let b = g.add_node((), SZ);
    let c = g.add_node((), SZ);
    let d = g.add_node((), SZ);
    g.add_edge(a, b);
    g.add_edge(a, c);
    g.add_edge(b, d);
    g.add_edge(c, d);
    g.add_edge(d, a);

    g.layout();
}
