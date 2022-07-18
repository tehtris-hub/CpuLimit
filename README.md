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


## License

Developped at [TEHTRIS](https://tehtris.com) by Fabien Savy.

Licensed under the GNU Lesser General Public License (LGPL) ,Version 3.0 (the "License"); you may not use this file except in compliance with the License.

You may obtain a copy of the License at [http://www.gnu.org/licenses/lgpl-3.0.txt](http://www.gnu.org/licenses/lgpl-3.0.txt])

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

See the License for the specific language governing permissions and limitations under the License.
