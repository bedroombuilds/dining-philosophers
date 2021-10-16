use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct Philosopher {
    name: String,
    left: usize,
    right: usize,
}

impl Philosopher {
    fn new(name: &str, left: usize, right: usize) -> Philosopher {
        Philosopher {
            name: name.to_string(),
            left,
            right,
        }
    }

    fn eat(&mut self, table: &Table) {
        println!("{} is asking waiter.", self.name);
        'eat: loop {
            if table.take_fork(Fork::Left(self.left)) {
                loop {
                    // try to take second fork
                    if table.take_fork(Fork::Right(self.right)) {
                        println!("{} is eating.", self.name);

                        thread::sleep(Duration::from_millis(1000));

                        println!("{} is done eating.", self.name);
                        table.return_fork(self.left);
                        table.return_fork(self.right);
                        break 'eat;
                    }
                }
            }
        }
    }
}

enum Fork {
    Left(usize),
    Right(usize),
}
type Forks = Vec<bool>;
struct Table {
    waiter: Mutex<Forks>,
}

impl Table {
    /// Strategy:
    /// Hand out free forks when requested, but when only one free fork available give priority
    /// to fork for a right hand (by convention the second hand that asks for a fork)
    fn take_fork(&self, which: Fork) -> bool {
        let mut forks = self.waiter.lock().unwrap();
        let forks_available = forks.iter().filter(|x| !*x).count();
        let mut reserve_fork = |fork: usize| {
            if !forks[fork] {
                forks[fork] = true;
                true
            } else {
                false
            }
        };
        match which {
            Fork::Right(fork) => reserve_fork(fork),
            Fork::Left(fork) => {
                if forks_available > 1 {
                    reserve_fork(fork)
                } else {
                    false
                }
            }
        }
    }

    fn return_fork(&self, fork: usize) {
        let mut forks = self.waiter.lock().unwrap();
        forks[fork] = false;
    }
}

fn main() {
    let philosophers = vec![
        Philosopher::new("Judith Butler", 0, 1),
        Philosopher::new("Gilles Deleuze", 1, 2),
        Philosopher::new("Karl Marx", 2, 3),
        Philosopher::new("Emma Goldman", 3, 4),
        Philosopher::new("Michel Foucault", 4, 0),
    ];
    let table = Arc::new(Table {
        waiter: Mutex::new(
            (0..philosophers.len())
                .into_iter()
                .map(|_| false)
                .collect::<Forks>(),
        ),
    });

    let handles: Vec<_> = philosophers
        .into_iter()
        .map(|mut p| {
            let table = table.clone();

            thread::spawn(move || {
                p.eat(&table);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
