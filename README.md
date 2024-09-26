A simple set of async combinators, usable in a `no_std`, `no_alloc` environment.

## Technical Note

Because of it's simplicity, Woven doesn't implement granular wakers, so an executer has no way of knowing which task woke it. This usually leads to all the combined futures being polled again, regardless of which one actually woke the executor. It's up to you whether this is acceptable or not.

## Usage

### Join

```rust
use woven::Join;

cassette::block_on(async {
    let future1 = async { 1 };
    let future2 = async { 2 };
    let future3 = async { 3 };

    let result = (future1, future2, future3).join().await;
    assert_eq!(result, (1, 2, 3));
});
```

### Race

```rust
use woven::{Race, Either3};

cassette::block_on(async {
    let future1 = async { 1 };
    let future2 = async { 2 };
    let future3 = async { 3 };

    let result = (future1, future2, future3).race().await;
    assert_eq!(result, Either3::First(1)); // If multiple futures complete at the same time, the first one is returned.
});
```

### Race Same

```rust
use woven::RaceSame;

cassette::block_on(async {
    let future1 = async { 1 };
    let future2 = async { 2 };
    let future3 = async { 3 };

    let result = (future1, future2, future3).race_same().await;
    assert_eq!(result, 1); // If multiple futures complete at the same time, the first one is returned.
});
```
