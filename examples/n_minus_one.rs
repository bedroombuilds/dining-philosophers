use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use semaphore::Semaphore;

struct Philosopher {
    name: String,
    left: usize,
    right: usize,
}

impl Philosopher {
    fn new(name: &str, left: usize, right: usize) -> Philosopher {
        assert!(left != right);
        Philosopher {
            name: name.to_string(),
            // on purpose not Dijkstra solution
            left,
            right,
        }
    }

    fn eat(&self, table: &Table) {
        let _left = table.forks[self.left].lock().unwrap();
        //thread::sleep(Duration::from_millis(1));
        let _right = table.forks[self.right].lock().unwrap();

        println!("{} is eating.", self.name);

        thread::sleep(Duration::from_millis(1000));

        println!("{} is done eating.", self.name);
    }
}

struct Table {
    forks: Vec<Mutex<()>>,
}

fn main() {
    let table = Arc::new(Table {
        forks: vec![
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
        ],
    });

    let philosophers = vec![
        Philosopher::new("Baruch Spinoza", 0, 1),
        Philosopher::new("Gilles Deleuze", 1, 2),
        Philosopher::new("Karl Marx", 2, 3),
        Philosopher::new("Friedrich Nietzsche", 3, 4),
        Philosopher::new("Michel Foucault", 4, 0),
    ];
    let n_minus_one = philosophers.len() - 1;

    let sem = Arc::new(Semaphore::new(n_minus_one, ()));

    let handles: Vec<_> = philosophers
        .into_iter()
        .map(|p| {
            let table = table.clone();
            let sem = sem.clone();

            thread::spawn(move || loop {
                if sem.try_access().is_ok() {
                    p.eat(&table);
                    break;
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
