#![feature(generators, generator_trait, conservative_impl_trait, proc_macro)]

extern crate futures_await as futures;
extern crate crossbeam;

use std::ops::Generator;
use std::ops::GeneratorState;
use futures::prelude::*;
use crossbeam::sync::chase_lev;

struct AsyncGenerator<T>
where T: Generator
{
    generator: T,
    producer: chase_lev::Worker<GeneratorState<T::Yield, T::Return>>,
    consumer: chase_lev::Stealer<GeneratorState<T::Yield, T::Return>>
}

impl<T> AsyncGenerator<T>
where T: Generator
{
    fn new(generator: T) -> Self {
        let (mut producer, consumer) = chase_lev::deque();
        let mut async_gen = AsyncGenerator {
            generator: generator,
            producer: producer,
            consumer: consumer
        };
        async_gen.populate();
        async_gen
    }

    #[async]
    fn resume(&mut self) -> Result<Option<T::Yield>, ()> {
        loop {
            match self.consumer.steal() {
                chase_lev::Steal::Data(data) => match data {
                    GeneratorState::Yielded(value) => return Ok(Some(value)),
                    GeneratorState::Complete(_) => return Ok(None)
                }
            }
        }
    }

    #[async]
    fn populate(&mut self) -> Result<(), ()> {
        loop {
            let next_item = self.generator.resume();
            self.producer.push(next_item);
            match next_item {
                GeneratorState::Complete(_) => break
            }
        }
        Ok(())
    }
}

trait IntoAsync<T>
where T: Generator
{
    fn into_async(self) -> AsyncGenerator<T>;
}

impl<T> IntoAsync<T> for T
where T: Generator
{
    fn into_async(self) -> AsyncGenerator<T> {
        AsyncGenerator::new(self)
    }
}

impl<T> Iterator for AsyncGenerator<T>
where T: Generator
{
    type Item = T::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        let future = async_block! {
            await!(self.resume())
        };
        match future.wait() {
            Ok(Some(next_value)) => Some(next_value),
            Ok(None) => None,
            Err(_) => None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Generator;
    use IntoAsync;

    fn dummy_generator() -> impl Generator<Yield=u32, Return=()> {
        || {
            yield 1;
            yield 2;
            yield 3;
            return;
        }
    }

    #[test]
    fn it_works() {
        let test : Vec<_> = dummy_generator().into_async().collect();

        assert_eq!(vec![1u32, 2, 3], test);
    }
}
