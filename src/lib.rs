#![feature(generators, generator_trait, conservative_impl_trait, proc_macro)]

extern crate futures_await as futures;

use std::ops::Generator;
use futures::prelude::*;

struct AsyncGenerator<T>
where T: Generator
{
    generator: T,
    values: Vec<T::Yield>
}

impl<T> AsyncGenerator<T>
where T: Generator + 'static
{
    fn new(generator: T) -> Self {
        AsyncGenerator {
            generator: generator,
            values: Vec::new()
        }
    }

    #[async]
    fn resume(&mut self) -> Result<T::Yield, T::Return> {
        unimplemented!()
    }
}

trait IntoAsync<T>
where T: Generator
{
    fn into_async(self) -> AsyncGenerator<T>;
}

impl<T> IntoAsync<T> for T
where T: Generator + 'static
{
    fn into_async(self) -> AsyncGenerator<T> {
        AsyncGenerator::new(self)
    }
}

impl<T> Iterator for AsyncGenerator<T>
where T: Generator + 'static
{
    type Item = T::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        let future = async_block! {
            await!(self.resume())
        };
        match future.wait() {
            Ok(next_value) => Some(next_value),
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
