//! Function transformers used to hide actual type signatures of functions stored
//! with a Join Pattern and instead expose a generic interface that is easily stored.

use std::any::Any;
use std::sync::mpsc::Sender;

use crate::types::{functions, Message};

/// Function transformers for functions stored with unary Join Patterns.
pub(crate) mod unary {
    use super::*;

    /// Transform function of `SendJoinPattern` to use `Message` arguments.
    pub(crate) fn transform_send<F, T>(f: F) -> Box<impl functions::unary::FnBoxClone>
    where
        F: Fn(T) -> () + Send + Clone + 'static,
        T: Any + Send + 'static,
    {
        Box::new(move |arg: Message| {
            f(*arg.downcast::<T>().unwrap());
        })
    }

    /// Transform function of `RecvJoinPattern` to use `Message` arguments.
    pub(crate) fn transform_recv<F, R>(f: F) -> Box<impl functions::unary::FnBoxClone>
    where
        F: Fn() -> R + Send + Clone + 'static,
        R: Any + Send + 'static,
    {
        Box::new(move |return_sender: Message| {
            let return_sender = *return_sender.downcast::<Sender<R>>().unwrap();

            return_sender.send(f()).unwrap();
        })
    }

    /// Transform function of `BidirJoinPattern` to use `Message` arguments.
    pub(crate) fn transform_bidir<F, T, R>(f: F) -> Box<impl functions::unary::FnBoxClone>
    where
        F: Fn(T) -> R + Send + Clone + 'static,
        T: Any + Send + 'static,
        R: Any + Send + 'static,
    {
        Box::new(move |arg_and_sender: Message| {
            let (arg, return_sender) = *arg_and_sender.downcast::<(T, Sender<R>)>().unwrap();

            return_sender.send(f(arg)).unwrap();
        })
    }
}

/// Function transformers for functions stored with binary Join Patterns.
pub(crate) mod binary {
    use super::*;

    /// Transform function of `SendJoinPattern` to use `Message` arguments.
    pub(crate) fn transform_send<F, T, U>(f: F) -> Box<impl functions::binary::FnBoxClone>
    where
        F: Fn(T, U) -> () + Send + Clone + 'static,
        T: Any + Send + 'static,
        U: Any + Send + 'static,
    {
        Box::new(move |arg_1: Message, arg_2: Message| {
            f(
                *arg_1.downcast::<T>().unwrap(),
                *arg_2.downcast::<U>().unwrap(),
            );
        })
    }

    /// Transform function of `RecvJoinPattern` to use `Message` arguments.
    pub(crate) fn transform_recv<F, T, R>(f: F) -> Box<impl functions::binary::FnBoxClone>
    where
        F: Fn(T) -> R + Send + Clone + 'static,
        T: Any + Send + 'static,
        R: Any + Send + 'static,
    {
        Box::new(move |arg: Message, return_sender: Message| {
            let return_sender = *return_sender.downcast::<Sender<R>>().unwrap();
            let arg = *arg.downcast::<T>().unwrap();

            return_sender.send(f(arg)).unwrap();
        })
    }

    /// Transform function of `BidirJoinPattern` to use `Message` arguments.
    pub(crate) fn transform_bidir<F, T, U, R>(f: F) -> Box<impl functions::binary::FnBoxClone>
    where
        F: Fn(T, U) -> R + Send + Clone + 'static,
        T: Any + Send + 'static,
        U: Any + Send + 'static,
        R: Any + Send + 'static,
    {
        Box::new(move |arg_1: Message, arg_2_and_sender: Message| {
            let arg_1 = *arg_1.downcast::<T>().unwrap();
            let (arg_2, return_sender) = *arg_2_and_sender.downcast::<(U, Sender<R>)>().unwrap();

            return_sender.send(f(arg_1, arg_2)).unwrap();
        })
    }
}

/// Function transformers for functions stored with ternary `JoinPattern`s.
pub(crate) mod ternary {
    use super::*;

    /// Transform function of `SendJoinPattern` to use `Message` arguments.
    pub(crate) fn transform_send<F, T, U, V>(f: F) -> Box<impl functions::ternary::FnBoxClone>
    where
        F: Fn(T, U, V) -> () + Send + Clone + 'static,
        T: Any + Send + 'static,
        U: Any + Send + 'static,
        V: Any + Send + 'static,
    {
        Box::new(move |arg_1: Message, arg_2: Message, arg_3: Message| {
            f(
                *arg_1.downcast::<T>().unwrap(),
                *arg_2.downcast::<U>().unwrap(),
                *arg_3.downcast::<V>().unwrap(),
            );
        })
    }

    /// Transform function of `RecvJoinPattern` to use `Message` arguments.
    pub(crate) fn transform_recv<F, T, U, R>(f: F) -> Box<impl functions::ternary::FnBoxClone>
    where
        F: Fn(T, U) -> R + Send + Clone + 'static,
        T: Any + Send + 'static,
        U: Any + Send + 'static,
        R: Any + Send + 'static,
    {
        Box::new(
            move |arg_1: Message, arg_2: Message, return_sender: Message| {
                let return_sender = *return_sender.downcast::<Sender<R>>().unwrap();
                let arg_1 = *arg_1.downcast::<T>().unwrap();
                let arg_2 = *arg_2.downcast::<U>().unwrap();

                return_sender.send(f(arg_1, arg_2)).unwrap();
            },
        )
    }

    /// Transform function of `BidirJoinPattern` to use `Message` arguments.
    pub(crate) fn transform_bidir<F, T, U, V, R>(f: F) -> Box<impl functions::ternary::FnBoxClone>
    where
        F: Fn(T, U, V) -> R + Send + Clone + 'static,
        T: Any + Send + 'static,
        U: Any + Send + 'static,
        V: Any + Send + 'static,
        R: Any + Send + 'static,
    {
        Box::new(
            move |arg_1: Message, arg_2: Message, arg_3_and_sender: Message| {
                let arg_1 = *arg_1.downcast::<T>().unwrap();
                let arg_2 = *arg_2.downcast::<U>().unwrap();
                let (arg_3, return_sender) =
                    *arg_3_and_sender.downcast::<(V, Sender<R>)>().unwrap();

                return_sender.send(f(arg_1, arg_2, arg_3)).unwrap();
            },
        )
    }
}
