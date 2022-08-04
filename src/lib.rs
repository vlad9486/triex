//! TriEx is a trivial executor. It executes a future on a current thread.

#![deny(clippy::all)]

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    thread,
};

/// Run the future and block the current thread until the future complete.
/// Does not waste CPU time.
pub fn run<F>(mut future: F) -> F::Output
where
    F: Future,
{
    let this = thread::current();

    let waker = waker_fn::waker_fn(move || this.unpark());
    let mut cx = Context::from_waker(&waker);
    let mut future = unsafe { Pin::new_unchecked(&mut future) };
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Pending => thread::park(),
            Poll::Ready(v) => break v,
        }
    }
}

#[cfg(test)]
#[test]
fn basic() {
    use std::{
        thread,
        time::{Duration, Instant},
    };

    struct F(bool);

    impl Future for F {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let waker = cx.waker().clone();
            if !self.0 {
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(1));
                    waker.wake()
                });
                self.0 = true;
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }

    let start = Instant::now();
    run(F(false));

    assert!((start.elapsed().as_secs_f64() - 1.0).abs() < 0.01f64);
}
