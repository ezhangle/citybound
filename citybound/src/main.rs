#![allow(dead_code)]
#[macro_use]
extern crate kay;
extern crate monet;
extern crate nalgebra;
extern crate compass;

use monet::glium::DisplayBuild;
use monet::glium::glutin;
use kay::{ID, Known, Message, Recipient, SystemServices, EVec, ActorSystem, Swarm, Inbox, MemChunker, Embedded};

#[derive(Copy, Clone)]
struct LaneCar {
    trip: ID,
    position: f32
}

derive_embedded!{
    struct Lane {
        length: f32,
        next: Option<ID>,
        previous: Option<ID>,
        cars: EVec<LaneCar>
    }
}
impl Known for Lane {fn type_id() -> usize {13}}

#[derive(Copy, Clone)]
struct AddCar(LaneCar);

impl Message for AddCar {}
impl Known for AddCar {fn type_id() -> usize {42}}

#[derive(Copy, Clone)]
struct Tick;
impl Message for Tick {}
impl Known for Tick {fn type_id() -> usize {43}}

impl Recipient<AddCar> for Lane {
    fn receive(&mut self, message: &AddCar, _system: &mut SystemServices) {
        self.cars.push(message.0);
    }
}

impl Recipient<Tick> for Lane {
    fn receive(&mut self, _message: &Tick, system: &mut SystemServices) {
        for car in &mut self.cars {
            car.position += 1.0;
        }
        while self.cars.len() > 0 {
            let mut last_car = self.cars[self.cars.len() - 1];
            if last_car.position > self.length {
                last_car.position -= self.length;
                system.send(AddCar(last_car), self.next.unwrap());
                self.cars.pop();
            } else {break;}
        }
    }
}

fn main() {
    
    let window = glutin::WindowBuilder::new()
        .with_title("Citybound".to_string())
        .with_dimensions(512, 512)
        .with_multitouch()
        .with_vsync().build_glium().unwrap();

    let mut system = ActorSystem::new();

    system.add_swarm::<Lane>(Swarm::new(MemChunker::new("lane_actors", 512), 30));    
    system.add_inbox::<AddCar, Lane>(Inbox::new(MemChunker::new("add_car", 512), 4));
    system.add_inbox::<Tick, Lane>(Inbox::new(MemChunker::new("tick", 512), 4));

    let (actor1, actor2) = {
        let swarm = system.swarm::<Lane>();

        let mut actor1 = swarm.create(Lane {
            length: 15.0,
            previous: None,
            next: None,
            cars: EVec::new()
        });

        let actor2 = swarm.create(Lane {
            length: 10.0,
            previous: Some(actor1.id),
            next: Some(actor1.id),
            cars: EVec::new()
        });

        actor1.next = Some(actor2.id);

        swarm.add(&actor1);
        swarm.add(&actor2);

        (actor1, actor2)
    };

    system.send(AddCar(LaneCar{position: 2.0, trip: ID::invalid()}), actor1.id);
    system.send(AddCar(LaneCar{position: 1.0, trip: ID::invalid()}), actor1.id);


    'main: loop {
        
        system.send(Tick, actor1.id);
        system.send(Tick, actor2.id);

        for _i in 0..1000 {
            system.process_messages();
        }

        {
            let swarm = system.swarm::<Lane>();
            println!("{}, {}", swarm.at(0).cars.len(), swarm.at(1).cars.len());
            println!("done!");
        }

        println!("rendering...");

    }
}