# Dining philosophers problem - Solutions implemented in Rust

You can read up on the Problem and its solutions at wikipedia <https://en.wikipedia.org/wiki/Dining_philosophers_problem>

Build all examples using `cargo build --release --examples`

## Resource hierarchy solution

`cargo run --release --example dijkstra`

In the constructor of a philosopher we make sure to try to access the lower numbered resource (the fork) first. This way a dead-lock is avoided even though two Mutex guards are used, one for each resource.

## Arbitrator solution

`cargo run --release --example arbitrator`

A "waiter" behind a single Mutex guard hands out forks to the philosophers on request. Requests are fulfilled until the waiter only has one fork left. Then only a philosopher requesting a fork for his right hand will be awarded one, avoiding a dead-lock.

## Limiting the number of diners in the table

`cargo run --release --example n_minus_one`

A semaphore is used to limit the amount of philosophers to n - 1. This way at least one philosopher can eat guaranteeing progress.

## Chandy/Misra solution

`cargo run --release --example chandy_misra`

Literal translation of solution into code using mpsc (multi-producer, single consumer) channels. Likely not efficient but good introduction on how inter-thread communication works.
