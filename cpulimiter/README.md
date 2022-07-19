# cpulimiter

Limit the CPU usage of a process.

## Example

```rust
use cpulimiter::Pid;

// Only limit the target process
Pid::from(1048).limit(10.0);
// or also account for the children
Pid::try_from("2096").unwrap().limit_with_children(42.0);
```
