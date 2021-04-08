
use std::{collections::HashMap, any::TypeId};
// use wgx::refs::{Flr, Id};



pub trait EventEmitter<'a, E, T, R> {
    fn register_event(&mut self, target:T, handler:Box<dyn 'a + FnMut(E)->R >);
    fn unregister_event(&mut self, target:T);
}

type Event = usize;
type Element = usize;
type Handler<'a> = Box<dyn 'a + FnMut(Event)-> bool>;

type Element1 = f32;


struct Dmm<'a> {
    nodes: HashMap<(Element, TypeId), (Element, Handler<'a>)>
}


impl<'a> EventEmitter<'a, Event, Element, bool> for Dmm<'a> {
    fn register_event(&mut self, node:Element, handler:Handler<'a>) {
        self.nodes.insert((node, TypeId::of::<Element>()), (node, handler));
    }
    fn unregister_event(&mut self, node:Element) {
        self.nodes.remove(&(node, TypeId::of::<Element>()));
    }
}

impl<'a> EventEmitter<'a, Event, Element1, bool> for Dmm<'a> {
    fn register_event(&mut self, node:Element1, handler:Handler<'a>) {
        self.nodes.insert((node as Element, TypeId::of::<Element1>()), (node as Element, handler));
    }
    fn unregister_event(&mut self, node:Element1) {
        self.nodes.remove(&(node as Element, TypeId::of::<Element1>()));
    }
}




impl Dmm<'_> {
    fn new() -> Self { Self { nodes: HashMap::new() } }

    fn dispatch_event(&mut self, event:Event) {
        self.nodes.iter_mut().for_each(|(_key, handler)| {
            handler.1(event);
        });
    }
}





fn main() {

    let mut dm = Dmm::new();

    let e1 = 34;
    let e2 = 44.0;


    dm.register_event(e1, Box::new(move |evt| {
        println!("{:?}: {:?}", e1, evt);
        true
    }));

    dm.register_event(e2, Box::new(move |evt| {
        println!("{:?}: {:?}", e2, evt);
        false
    }));

    dm.dispatch_event(122 as Event);
}
