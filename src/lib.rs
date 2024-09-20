//! Future combinators and utilities.

#![no_std]

use core::future::Future;

/// Combine multiple futures into one that resolves when all are done.
pub trait Join {
    /// The output type of the combined future.
    type Output;

    /// Combine multiple futures into one that resolves when all are done.
    fn join(self) -> impl Future<Output = Self::Output>;
}

/// Combine multiple futures into one that resolves when any single one is done.
pub trait Race {
    /// The output type of the combined future.
    type Output;

    /// Combine multiple futures into one that resolves when any single one is
    /// done.
    fn race(self) -> impl Future<Output = Self::Output>;
}

/// Combine multiple futures with the same output into one that resolves when
/// any single one is done.
pub trait RaceSame {
    /// The output type of the combined future.
    type Output;

    /// Combine multiple futures with the same output into one that resolves
    /// when any single one is done.
    fn race_same(self) -> impl Future<Output = Self::Output>;
}

enum MaybeDone<Fut: Future> {
    /// A not-yet-completed future, must be pinned.
    Future(Fut),
    /// The output of the completed future
    Done(Fut::Output),
    /// Empty variant after data has been taken.
    Gone,
}

impl<Fut: Future + Unpin> Unpin for MaybeDone<Fut> {}

impl<Fut: Future> MaybeDone<Fut> {
    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };

        match this {
            Self::Future(fut) => match unsafe { core::pin::Pin::new_unchecked(fut) }.poll(cx) {
                core::task::Poll::Ready(res) => {
                    *this = Self::Done(res);
                    true
                }
                core::task::Poll::Pending => false,
            },
            _ => true,
        }
    }

    fn take_output(&mut self) -> Fut::Output {
        match &*self {
            Self::Done(_) => {}
            Self::Future(_) | Self::Gone => unreachable!(),
        }

        match core::mem::replace(self, Self::Gone) {
            MaybeDone::Done(output) => output,
            _ => unreachable!(),
        }
    }
}

macro_rules! impl_combinators {
    (
        $Either: ident, $( $F: ident : $Nth: ident ),*
    ) => {
        impl< $( $F ),* > Join for ( $( $F ),* )
        where
            $( $F: Future ),*
        {
            type Output = ( $( $F::Output ),* );

            fn join(self) -> impl Future<Output = Self::Output> {
                #[allow(non_snake_case)]
                struct Join< $( $F: Future ),* > {
                    $( $F: MaybeDone<$F> ),*
                }

                impl< $( $F ),* > Future for Join< $( $F ),* >
                where
                    $( $F: Future ),*
                {
                    type Output = ( $( $F::Output ),* );

                    fn poll(
                        self: core::pin::Pin<&mut Self>,
                        cx: &mut core::task::Context<'_>,
                    ) -> core::task::Poll<Self::Output> {
                        let this = unsafe { self.get_unchecked_mut() };
                        let mut done = true;
                        $(
                            done &= unsafe { core::pin::Pin::new_unchecked(&mut this.$F) }.poll(cx);
                        )*
                        if done {
                            core::task::Poll::Ready(($( this.$F.take_output(), )*))
                        } else {
                            core::task::Poll::Pending
                        }
                    }
                }

                #[allow(non_snake_case)]
                let ( $( $F ),* ) = self;

                Join {
                    $( $F: MaybeDone::Future( $F ) ),*
                }
            }
        }

        /// An enum representing the output of a [`Race`] operation.
        pub enum $Either< $( $F ),* > {
            $(
                #[doc = concat!("The ", stringify!($Nth), " possible value.")]
                $Nth( $F ),
            )*
        }

        impl< $( $F ),* > Race for ( $( $F ),* )
        where
            $( $F: Future ),*
        {
            type Output = $Either< $( $F::Output ),* >;

            async fn race(self) -> Self::Output {
                #[allow(non_snake_case)]
                let ( $( $F ),* ) = self;

                $(
                    #[allow(non_snake_case)]
                    let mut $F = core::pin::pin!($F);
                )*

                core::future::poll_fn(move |cx| {
                    $(
                        if let core::task::Poll::Ready(x) = $F.as_mut().poll(cx) {
                            return core::task::Poll::Ready($Either::$Nth(x));
                        }
                    )*

                    core::task::Poll::Pending
                })
                .await
            }
        }

        impl<T, $( $F ),* > RaceSame for ( $( $F ),* )
        where
            $( $F: Future<Output = T> ),*
        {
            type Output = T;

            async fn race_same(self) -> Self::Output {
                #[allow(non_snake_case)]
                let ( $( $F ),* ) = self;

                $(
                    #[allow(non_snake_case)]
                    let mut $F = core::pin::pin!($F);
                )*

                core::future::poll_fn(move |cx| {
                    $(
                        if let core::task::Poll::Ready(x) = $F.as_mut().poll(cx) {
                            return core::task::Poll::Ready(x);
                        }
                    )*

                    core::task::Poll::Pending
                })
                .await
            }
        }
    };
}

impl_combinators!(Either, F0: First, F1: Second);
impl_combinators!(Either3, F0: First, F1: Second, F2: Third);
impl_combinators!(Either4, F0: First, F1: Second, F2: Third, F3: Fourth);
impl_combinators!(Either5, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth);
impl_combinators!(Either6, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth);
impl_combinators!(Either7, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh);
impl_combinators!(Either8, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh, F7: Eighth);
impl_combinators!(Either9, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh, F7: Eighth, F8: Ninth);
impl_combinators!(Either10, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh, F7: Eighth, F8: Ninth, F9: Tenth);
impl_combinators!(Either11, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh, F7: Eighth, F8: Ninth, F9: Tenth, F10: Eleventh);
impl_combinators!(Either12, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh, F7: Eighth, F8: Ninth, F9: Tenth, F10: Eleventh, F11: Twelfth);
impl_combinators!(Either13, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh, F7: Eighth, F8: Ninth, F9: Tenth, F10: Eleventh, F11: Twelfth, F12: Thirteenth);
impl_combinators!(Either14, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh, F7: Eighth, F8: Ninth, F9: Tenth, F10: Eleventh, F11: Twelfth, F12: Thirteenth, F13: Fourteenth);
impl_combinators!(Either15, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh, F7: Eighth, F8: Ninth, F9: Tenth, F10: Eleventh, F11: Twelfth, F12: Thirteenth, F13: Fourteenth, F14: Fifteenth);
impl_combinators!(Either16, F0: First, F1: Second, F2: Third, F3: Fourth, F4: Fifth, F5: Sixth, F6: Seventh, F7: Eighth, F8: Ninth, F9: Tenth, F10: Eleventh, F11: Twelfth, F12: Thirteenth, F13: Fourteenth, F14: Fifteenth, F15: Sixteenth);
