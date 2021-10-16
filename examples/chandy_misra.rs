//! Description from Wikipedia
//!
//! 1. For every pair of philosophers contending for a resource, create a fork and give it to the philosopher with the lower ID (n for agent Pn). Each fork can either be dirty or clean. Initially, all forks are dirty.
//! 2. When a philosopher wants to use a set of resources (i.e., eat), said philosopher must obtain the forks from their contending neighbors. For all such forks the philosopher does not have, they send a request message.
//! 3. When a philosopher with a fork receives a request message, they keep the fork if it is clean, but give it up when it is dirty. If the philosopher sends the fork over, they clean the fork before doing so.
//! 4. After a philosopher is done eating, all their forks become dirty. If another philosopher had previously requested one of the forks, the philosopher that has just finished eating cleans the fork and sends it.

use std::cell::Cell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq)]
enum ForkState {
    Dirty,
    Clean,
}

struct Fork {
    id: usize,
    state: ForkState,
}

impl std::fmt::Debug for Fork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self.state {
            ForkState::Dirty => "dirty",
            ForkState::Clean => "clean",
        };
        f.write_fmt(format_args!("Fork {} {}", self.id, state))
    }
}

impl Fork {
    fn new_dirty(id: usize) -> Self {
        Self {
            id,
            state: ForkState::Dirty,
        }
    }
    fn is_dirty(&self) -> bool {
        self.state == ForkState::Dirty
    }
    fn clean(&mut self) {
        self.state = ForkState::Clean;
    }
    fn dirty(&mut self) {
        self.state = ForkState::Dirty;
    }
}

struct ForkStorage {
    requested: Cell<bool>,
    fork: Option<Fork>,
}

impl std::fmt::Debug for ForkStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let requested = if self.requested.get() {
            "requested"
        } else {
            "not-req"
        };
        match &self.fork {
            Some(fork) => {
                let state = match fork.state {
                    ForkState::Dirty => "dirty",
                    ForkState::Clean => "clean",
                };
                f.write_fmt(format_args!("Fork {} {} {}", fork.id, state, requested))
            }
            None => f.write_fmt(format_args!("Fork None {}", requested)),
        }
    }
}

impl ForkStorage {
    fn new(fork: Option<Fork>) -> Self {
        Self {
            requested: Cell::new(false),
            fork,
        }
    }

    fn is_some(&self) -> bool {
        self.fork.is_some()
    }

    fn is_dirty(&self) -> bool {
        self.fork.as_ref().map_or(false, |f| f.is_dirty())
    }

    fn take(&mut self) -> Option<Fork> {
        self.fork.take()
    }

    fn dirty(&mut self) {
        self.fork.as_mut().unwrap().dirty();
    }

    fn needs_requesting(&self) -> bool {
        self.fork.is_none() && !self.requested.get()
    }

    fn requested(&self) {
        self.requested.set(true);
    }
}

enum ForkMessage {
    Request(usize, usize),
    Delivery(Fork),
}

#[derive(Debug)]
struct Philosopher {
    id: usize,
    name: String,
    receiver: mpsc::Receiver<ForkMessage>,
    neighbours: HashMap<usize, mpsc::Sender<ForkMessage>>,
    forks: BTreeMap<usize, ForkStorage>,
    request_queue: VecDeque<(usize, usize)>,
}

impl Philosopher {
    fn new(id: usize, name: &str, receiver: mpsc::Receiver<ForkMessage>) -> Self {
        Self {
            id,
            name: name.to_string(),
            receiver,
            neighbours: HashMap::new(),
            forks: BTreeMap::new(),
            request_queue: VecDeque::new(),
        }
    }

    fn setup_sender(&mut self, phil_id: usize, sender: mpsc::Sender<ForkMessage>) {
        self.neighbours.insert(phil_id, sender);
    }

    /// take fork into left hand first, subsequent fork goes to right
    fn setup_fork(&mut self, neighbour: usize, fork: Option<Fork>) {
        assert!(self.forks.len() <= 2, "Someone gave me a third fork");
        self.forks.insert(neighbour, ForkStorage::new(fork));
    }

    fn eat(&mut self) {
        // eat when both forks are available
        if self.forks.iter().all(|(_, f)| f.is_some()) {
            println!("{} {} is eating. {:?}", self.id, self.name, self.forks);
            thread::sleep(Duration::from_secs(1));
            // make forks dirty
            for (_pid, fork) in self.forks.iter_mut() {
                fork.dirty();
            }
            println!("{} {} is done eating. {:?}", self.id, self.name, self.forks);
            // give forks to others who requested them
            while let Some((who, fork_id)) = self.request_queue.pop_front() {
                if self.forks[&fork_id].is_dirty() {
                    let mut fork = self.forks.get_mut(&fork_id).unwrap().take().unwrap();
                    fork.clean();
                    let _ = self.neighbours[&who].send(ForkMessage::Delivery(fork));
                }
            }
        } else {
            // cannot eat, request the lower-numbered missing resource at neighbours
            self.forks
                .iter() // BTreeMap ordered by key
                .filter(|(_, f)| f.needs_requesting())
                .take(1)
                .for_each(|(fid, fork)| {
                    let pid = if *fid == self.id {
                        self.neighbour_left()
                    } else {
                        self.neighbour_right()
                    };
                    let _ = self.neighbours[&pid].send(ForkMessage::Request(self.id, *fid));
                    fork.requested();
                });
        }
    }

    fn neighbour_left(&self) -> usize {
        if self.id == 0 {
            *self.neighbours.keys().max().unwrap()
        } else {
            self.id - 1
        }
    }

    fn neighbour_right(&self) -> usize {
        *self
            .neighbours
            .keys()
            .find(|x| **x != self.neighbour_left())
            .unwrap()
    }

    /// go over messages, store or hand out forks
    fn handle_requests(&mut self) {
        if let Ok(msg) = self.receiver.recv() {
            match msg {
                ForkMessage::Request(by, fork_id) => {
                    if self.forks[&fork_id].is_dirty() {
                        let mut fork = self.forks.get_mut(&fork_id).unwrap().take().unwrap();
                        fork.clean();
                        let _ = self.neighbours[&by].send(ForkMessage::Delivery(fork));
                    } else {
                        // no fork or is clean queue for later
                        if !self.request_queue.iter().any(|&x| x == (by, fork_id)) {
                            self.request_queue.push_back((by, fork_id));
                        }
                    }
                }
                ForkMessage::Delivery(fork) => {
                    self.forks.insert(fork.id, ForkStorage::new(Some(fork)));
                }
            }
        }
    }

    fn run(&mut self) {
        loop {
            self.eat();
            self.handle_requests();
        }
    }
}

fn main() {
    let names = vec![
        "Baruch Spinoza",
        "Gilles Deleuze",
        "Karl Marx",
        "Friedrich Nietzsche",
        "Michael Foucault",
    ];
    let mut senders = vec![];
    let mut phils: Vec<Philosopher> = names
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let (s, r) = mpsc::channel::<ForkMessage>();
            senders.push(s);
            Philosopher::new(i, n, r)
        })
        .collect();

    let num_phils = phils.len();
    let phil_max_idx = num_phils - 1;
    for (i, sender) in senders.iter().enumerate() {
        let neighbour_left = if i == 0 { phil_max_idx } else { i - 1 };
        let neighbour_right = if i == phil_max_idx { 0 } else { i + 1 };
        phils[neighbour_left].setup_sender(i, sender.clone());
        phils[neighbour_right].setup_sender(i, sender.clone());
        let gets_fork = std::cmp::min(neighbour_left, i);
        phils[i].setup_fork(i, None);
        phils[i].setup_fork(neighbour_right, None);
        phils[gets_fork].setup_fork(i, Some(Fork::new_dirty(i)));
    }
    let mut handles = vec![];
    for mut p in phils {
        handles.push(thread::spawn(move || {
            p.run();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
