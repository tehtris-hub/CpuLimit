# cpulimiter

Limit the CPU usage of a process.

## Example

```rust
use cpulimiter::{CpuLimit, Pid};

let handle = CpuLimit::new(Pid::from(1048), 10.0).unwrap();
handle.set_limit(42.0);
handle.stop();
```
