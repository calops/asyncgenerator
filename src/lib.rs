#![feature(generators, generator_trait, conservative_impl_trait, proc_macro)]

extern crate crossbeam;
extern crate futures_await as futures;

use std::ops::Generator;
use std::ops::GeneratorState;
use futures::prelude::*;
use crossbeam::sync::chase_lev;

struct AsyncGenerator<T>
where
    T: Generator,
{
    stealer: chase_lev::Stealer<GeneratorState<T::Yield, T::Return>>,
}

impl<T> AsyncGenerator<T>
where
    T: Generator + 'static,
{
    fn new(generator: T) -> Self {
        let (worker, stealer) = chase_lev::deque();
        populate(generator, worker);
        AsyncGenerator { stealer: stealer }
    }

    fn resume(&mut self) -> Option<T::Yield> {
        loop {
            match self.stealer.steal() {
                chase_lev::Steal::Data(data) => match data {
                    GeneratorState::Yielded(value) => return Some(value),
                    GeneratorState::Complete(_) => return None,
                },
                chase_lev::Steal::Empty => continue,
                chase_lev::Steal::Abort => return None,
            }
        }
    }
}

#[async]
fn populate<T>(
    mut generator: T,
    worker: chase_lev::Worker<GeneratorState<T::Yield, T::Return>>,
) -> Result<(), ()>
where
    T: Generator + 'static,
{
    loop {
        match generator.resume() {
            next_item @ GeneratorState::Yielded(_) => worker.push(next_item),
            next_item => {
                worker.push(next_item);
                return Ok(());
            }
        }
    }
}

trait IntoAsync<T>
where
    T: Generator,
{
    fn into_async(self) -> AsyncGenerator<T>;
}

impl<T> IntoAsync<T> for T
where
    T: Generator + 'static,
{
    fn into_async(self) -> AsyncGenerator<T> {
        AsyncGenerator::new(self)
    }
}

impl<T> Iterator for AsyncGenerator<T>
where
    T: Generator + 'static,
{
    type Item = T::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        return self.resume();
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Generator;
    use IntoAsync;

    fn dummy_generator() -> impl Generator<Yield = u32, Return = ()> {
        || {
            yield 1;
            yield 2;
            yield 3;
            return;
        }
    }

    #[test]
    fn it_works() {
        let test: Vec<_> = dummy_generator().into_async().collect();

        assert_eq!(vec![1u32, 2, 3], test);
    }
}
