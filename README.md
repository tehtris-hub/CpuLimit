# CPUlimit

An oxidized version of the original [`cpulimit`](https://github.com/opsengine/cpulimit),
a utility to limit the CPU usage of a process.

## Usage

Limit process `4562` to 10%.

```console
cpulimit --pid 4562 --limit 10
```

Run `cpulimit --help` to list all the available options.

## Design

This crate implements user-space scheduling: after each time slice (currently 100 ms),
`cpulimit` wakes up and parses the `/proc/<pid>/stat` file to check how long the target process ran.
It then sends the `SIGSTOP` and `SIGCONT` signals to suspend and resume execution in order to
obtain the desired CPU usage.

## Limitations

- `cpulimit` only supports Linux-based operating systems.
- only single-threaded processes are currently supported.
