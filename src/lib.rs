#![feature(generators, generator_trait, conservative_impl_trait, proc_macro)]

extern crate futures_await as futures;

use std::ops::{Generator, GeneratorState};
use futures::prelude::{async, await};

struct AsyncGenerator<T>
where T: Generator
{
    generator: T
}

impl<T> AsyncGenerator<T>
where T: Generator
{
    fn new(generator: T) -> Self {
        AsyncGenerator {
            generator: generator
        }
    }
    
    #[async]
    fn resume(&mut self) -> GeneratorState<T::Yield, T::Return> {
        unimplemented!()
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
        match await!(self.resume()) {
            Ok(GeneratorState::Yielded(next_item))=> next_item,
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    fn dummy_generator() -> impl Generator {
        || {
            yield 1;
            yield 2;
            yield 3;
            return "done";
        }
    }

    #[test]
    fn it_works() {
        let test : Vec<_> = dummy_generator().into_async().collect();
    }
}
