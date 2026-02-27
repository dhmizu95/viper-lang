# Core Modules Implementation Status

**Document Version:** 1.0  
**Last Updated:** 2026-02-27  
**Viper Version:** 0.4.1

---

## Overview

This document tracks the implementation status of the 20 core standard library modules for Viper, as outlined in `docs/CORE_MODULES.md`. The implementation follows a three-layer pattern:

1. **C Runtime** (`runtime/src/*.c`) - Low-level implementations linked into `libviper.a`
2. **Viper Stdlib** (`std/core/*.vp`) - High-level wrappers and domain logic
3. **Compiler Wiring** (`src/jit_stubs/*.rs`) - JIT execution bindings

---

## Implementation Status Summary

| Phase | Module | C Runtime | Viper Wrapper | JIT Stubs | Status |
|-------|--------|-----------|---------------|-----------|--------|
| **Phase 1** | sys | ✅ | ✅ | ✅ | **Complete** |
| **Phase 1** | os | ✅ | ✅ | ✅ | **Complete** |
| **Phase 1** | time | ✅ | ✅ | ✅ | **Complete** |
| **Phase 1** | gc | ✅ | ✅ | ✅ | **Complete** |
| **Phase 2** | math | ✅ | ✅ | ⏳ | **Runtime Complete** |
| **Phase 2** | json | ✅ | ✅ | ⏳ | **Runtime Complete** |
| **Phase 2** | collections | ✅ | ✅ | ⏳ | **Runtime Complete** |
| **Phase 2** | re (regex) | ✅ | ✅ | ⏳ | **Runtime Complete** |
| **Phase 2** | random | ✅ | ✅ | ⏳ | **Runtime Complete** |
| **Phase 3** | socket | ✅ | ⏳ | ⏳ | **Runtime Only** |
| **Phase 3** | asyncio | ✅ | ⏳ | ⏳ | **Runtime Only** |
| **Phase 3** | http | ✅ | ⏳ | ⏳ | **Runtime Only** |
| **Phase 3** | select | ✅ | ⏳ | ⏳ | **Runtime Only** |
| **Phase 4** | hashlib | ✅ | ⏳ | ⏳ | **Runtime Only** |
| **Phase 4** | decimal | ✅ | ⏳ | ⏳ | **Runtime Only** |
| **Phase 4** | logging | ✅ | ⏳ | ⏳ | **Runtime Only** |
| **Phase 4** | pathlib | N/A | ✅ | N/A | **Pure Viper** |
| **Phase 4** | argparse | N/A | ✅ | N/A | **Pure Viper** |

**Legend:** ✅ Complete | ⏳ Pending | N/A Pure Viper (no C runtime)

---

## Phase 1: Runtime Foundation ✅ COMPLETE

### sys Module
**Files:** `runtime/src/sys.c`, `std/core/sys.vp`, `src/jit_stubs/sys.rs`

**Implemented Functions:**
- `vp_sys_exit(code)` - Exit program with status code
- `vp_sys_getpid()` - Get process ID
- `vp_sys_get_version()` - Get Viper version string
- `vp_sys_get_platform()` - Get platform identifier (linux/darwin/windows)
- `vp_sys_get_sysname()` - Get system name (uname -s)
- `vp_sys_get_machine()` - Get machine architecture
- `vp_sys_getenv(name)` - Get environment variable
- `vp_sys_setenv(name, value, overwrite)` - Set environment variable
- `vp_sys_unsetenv(name)` - Unset environment variable
- `vp_sys_init(argc, argv)` - Initialize with command-line args
- `vp_sys_get_argv()` - Get command-line arguments

**Viper API:**
```viper
import sys
sys.exit(0)
sys.getenv("HOME")
sys.platform  # "linux"
sys.version   # "0.4.1"
```

---

### os Module
**Files:** `runtime/src/os.c`, `std/core/os.vp`, `src/jit_stubs/os.rs`

**Implemented Functions:**
- `vp_os_getcwd()` - Get current working directory
- `vp_os_chdir(path)` - Change current directory
- `vp_os_listdir(path)` - List directory contents
- `vp_os_path_join(a, b)` - Join path components
- `vp_os_mkdir(path, mode)` - Create directory
- `vp_os_makedirs(path, mode)` - Create directories recursively
- `vp_os_remove(path)` - Remove file or directory
- `vp_os_path_exists(path)` - Check if path exists
- `vp_os_path_isfile(path)` - Check if path is a file
- `vp_os_path_isdir(path)` - Check if path is a directory
- `vp_os_path_getsize(path)` - Get file size in bytes
- `vp_os_path_abspath(path)` - Get absolute path
- `vp_os_path_basename(path)` - Get basename
- `vp_os_path_dirname(path)` - Get directory name
- `vp_os_rename(src, dst)` - Rename/move file
- `vp_os_copy(src, dst)` - Copy file
- `vp_os_get_home()` - Get user's home directory
- `vp_os_stat(path, ...)` - Get file statistics

**Viper API:**
```viper
import os
os.getcwd()
os.listdir(".")
os.mkdir("newdir")
os.path.join("/home", "user")
os.path.exists("/tmp")
```

---

### time Module
**Files:** `runtime/src/time_mod.c`, `std/core/time.vp`, `src/jit_stubs/time_mod.rs`

**Implemented Functions:**
- `vp_time_time()` - Get Unix timestamp (wall clock)
- `vp_time_monotonic()` - Get monotonic time
- `vp_time_perf_counter()` - Get performance counter
- `vp_time_sleep(seconds)` - Sleep for duration
- `vp_time_localtime(timestamp, ...)` - Convert to local time struct
- `vp_time_gmtime(timestamp, ...)` - Convert to UTC time struct
- `vp_time_strftime(timestamp, format)` - Format timestamp
- `vp_time_timezone_offset()` - Get timezone offset
- `vp_time_isdst()` - Check if DST is in effect
- `vp_time_days_in_month(year, month)` - Get days in month

**Viper API:**
```viper
import time
time.time()
time.sleep(1.5)
time.localtime()
time.strftime("%Y-%m-%d")
```

---

### gc Module
**Files:** `runtime/src/gc.c`, `std/core/gc.vp`, `src/jit_stubs/gc.rs`

**Implemented Functions:**
- `vp_gc_collect()` - Trigger garbage collection
- `vp_gc_disable()` - Disable automatic GC
- `vp_gc_enable()` - Enable automatic GC
- `vp_gc_is_enabled()` - Check if GC is enabled
- `vp_gc_get_count()` - Get collection count
- `vp_gc_get_total_freed()` - Get total bytes freed
- `vp_gc_get_memory_usage()` - Get current memory usage
- `vp_gc_set_threshold(bytes)` - Set GC threshold
- `vp_gc_get_stats()` - Get GC statistics string
- `vp_gc_print_stats()` - Print GC statistics
- `vp_gc_reset_stats()` - Reset statistics

**Viper API:**
```viper
import gc
gc.collect()
gc.disable()
gc.get_stats()
```

---

## Phase 2: Data & Performance ✅ RUNTIME COMPLETE

### math Module
**Files:** `runtime/src/math_mod.c`, `std/core/math.vp`

**Implemented Functions (60+):**
- **Constants:** `pi`, `e`, `tau`, `inf`, `nan`
- **Basic:** `sqrt`, `cbrt`, `floor`, `ceil`, `trunc`, `round`, `fabs`, `abs`
- **Power/Log:** `log`, `log2`, `log10`, `exp`, `exp2`, `pow`
- **Trigonometric:** `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`
- **Hyperbolic:** `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`
- **Angle Conversion:** `degrees`, `radians`
- **Rounding:** `fmod`, `remainder`, `fma`, `copysign`, `nextafter`
- **Min/Max:** `fmin`, `fmax`, `fdim`
- **Classification:** `isnan`, `isinf`, `isfinite`, `isnormal`, `signbit`
- **Special:** `erf`, `erfc`, `tgamma`, `lgamma`
- **Integer Math:** `gcd`, `lcm`, `factorial`, `comb`, `perm`
- **Distance:** `hypot`, `dist_2d`, `dist_3d`
- **Statistics:** `mean`, `variance`, `stddev`

**Viper API:**
```viper
import math
math.sqrt(2)
math.sin(math.pi / 2)
math.gcd(48, 18)
math.factorial(5)
```

---

### json Module
**Files:** `runtime/src/json.c`, `std/core/json.vp`

**Implemented Functions:**
- `vp_json_loads(json_str)` - Parse JSON string to dict
- `vp_json_dumps(dict)` - Convert dict to JSON string
- `vp_json_load_file(filename)` - Load JSON from file
- `vp_json_dump_file(dict, filename)` - Write JSON to file

**Features:**
- Recursive-descent parser (no external dependencies)
- Supports: strings, numbers, booleans, null, arrays, objects
- Escape sequence handling (including Unicode \uXXXX)
- LRU cache for compiled patterns (16 slots)

**Viper API:**
```viper
import json
data = json.loads('{"key": "value"}')
text = json.dumps(data)
```

---

### collections Module
**Files:** `runtime/src/collections.c`, `std/core/collections.vp`

**Implemented Types:**
- **ViperDeque** - Doubly-linked list
  - `append`, `appendleft`, `pop`, `popleft`, `rotate`, `clear`
- **ViperCounter** - Dict-backed frequency counter
  - `add`, `get`, `set`, `most_common`, `total`
- **ViperOrderedDict** - Dict with insertion order
  - `set`, `get`, `keys`, `values`, `items`, `move_to_end`
- **ViperDefaultDict** - Dict with default factory
- **ViperNamedTuple** - Struct-like container

**Viper API:**
```viper
from collections import deque, Counter, OrderedDict
d = deque([1, 2, 3])
c = Counter(["a", "b", "a"])
od = OrderedDict()
```

---

### re (Regex) Module
**Files:** `runtime/src/re_mod.c`, `std/core/re.vp`

**Implemented Functions:**
- `vp_re_compile(pattern, flags)` - Compile regex pattern
- `vp_re_match(pattern, string, pos)` - Match at beginning
- `vp_re_search(pattern, string, pos, endpos)` - Search anywhere
- `vp_re_findall(pattern, string)` - Find all matches
- `vp_re_split(pattern, string)` - Split by pattern
- `vp_re_sub(pattern, repl, string, count)` - Substitute matches
- `vp_re_fullmatch(pattern, string)` - Match entire string
- `vp_re_escape(string)` - Escape special characters

**Features:**
- POSIX regex wrappers (`<regex.h>`)
- LRU cache for compiled patterns (16 slots)
- Flags: IGNORECASE, MULTILINE, DOTALL, VERBOSE

**Viper API:**
```viper
import re
pattern = re.compile(r"\d+")
matches = re.findall(r"\w+", "hello world")
```

---

### random Module
**Files:** `runtime/src/random_mod.c`, `std/core/random.vp`

**Implemented Functions:**
- **Basic:** `random`, `randint`, `randrange`, `uniform`
- **Seeding:** `seed`, `seed_secure`, `getstate`, `setstate`
- **Sequences:** `choice`, `choices`, `shuffle`, `sample`
- **Distributions:** `gauss`, `normalvariate`, `expovariate`
- **Other:** `triangular`, `betavariate`, `gammavariate`, `weibullvariate`
- **Discrete:** `binomialvariate`, `geometricvariate`, `poissonvariate`

**Implementation:**
- PCG64 (Permuted Congruential Generator) algorithm
- Secure seeding from `/dev/urandom`
- Box-Muller transform for Gaussian distribution

**Viper API:**
```viper
import random
random.seed(42)
n = random.randint(1, 100)
items = random.sample([1, 2, 3, 4, 5], 3)
```

---

## Phase 3: Networking & Concurrency ✅ RUNTIME COMPLETE

### socket Module
**Files:** `runtime/src/socket_mod.c`

**Implemented Functions:**
- `vp_socket_create(family, type, protocol)` - Create socket
- `vp_socket_bind(sock, host, port)` - Bind to address
- `vp_socket_listen(sock, backlog)` - Listen for connections
- `vp_socket_connect(sock, host, port)` - Connect to server
- `vp_socket_accept(sock)` - Accept incoming connection
- `vp_socket_send(sock, data, len)` - Send data
- `vp_socket_sendall(sock, data, len)` - Send all data
- `vp_socket_recv(sock, buffer, maxlen)` - Receive data
- `vp_socket_sendto/recvfrom` - UDP operations
- `vp_socket_setblocking(sock, blocking)` - Set blocking mode
- `vp_socket_getpeername/sockname` - Get socket addresses
- `vp_socket_close(sock)` - Close socket

**Constants:**
- `AF_INET`, `AF_INET6`, `SOCK_STREAM`, `SOCK_DGRAM`
- `SOL_SOCKET`, `SO_REUSEADDR`, `TCP_NODELAY`
- `SHUT_RD`, `SHUT_WR`, `SHUT_RDWR`

**Viper API:**
```viper
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect(("localhost", 8080))
s.send(b"hello")
```

---

### asyncio Module
**Files:** `runtime/src/asyncio_mod.c`

**Implemented Functions:**
- `vp_asyncio_init()` - Initialize event loop
- `vp_asyncio_run(coroutine)` - Run async main
- `vp_asyncio_sleep(seconds)` - Async sleep
- `vp_asyncio_gather(coroutines)` - Run multiple coroutines
- **Task:** `create_task`, `task_done`, `task_cancel`, `task_result`
- **Lock:** `lock_create`, `lock_acquire`, `lock_release`
- **Event:** `event_create`, `event_set`, `event_wait`, `event_is_set`
- **Queue:** `queue_create`, `queue_put`, `queue_get`, `queue_size`
- **Semaphore:** `semaphore_create`, `semaphore_acquire/release`
- **Timeout:** `timeout_create`, `timeout_expired`

**Integration:**
- Adapters over existing `event_loop_epoll.c`
- Fiber-based concurrency support

**Viper API:**
```viper
import asyncio
async def main():
    await asyncio.sleep(1.0)
asyncio.run(main())
```

---

### http Module
**Files:** `runtime/src/http_mod.c`

**Implemented Functions:**
- **Client:**
  - `vp_http_get(url)` - HTTP GET request
  - `vp_http_post(url, body)` - HTTP POST request
  - `vp_http_request(method, url, body, headers)` - Generic request
- **Response:**
  - `vp_http_response_status(resp)` - Get status code
  - `vp_http_response_text(resp)` - Get body text
  - `vp_http_response_json(resp)` - Parse body as JSON
  - `vp_http_response_header(resp, name)` - Get header value
- **Server:**
  - `vp_http_server_create(port, handler)` - Create server
  - `vp_http_server_serve(server)` - Start serving
  - `vp_http_server_stop(server)` - Stop server
- **Utilities:**
  - `vp_http_urlencode/decode(string)` - URL encoding
  - `vp_http_build_response(...)` - Build HTTP response
  - `vp_http_parse_request(raw)` - Parse HTTP request

**Status Codes:**
- 200 OK, 201 Created, 204 No Content
- 301 Moved, 302 Found, 304 Not Modified
- 400 Bad Request, 401 Unauthorized, 403 Forbidden
- 404 Not Found, 405 Method Not Allowed
- 500 Internal Error, 501 Not Implemented, 503 Unavailable

**Viper API:**
```viper
import http
resp = http.get("https://api.example.com/data")
print(resp.status_code, resp.text)
```

---

### select Module
**Files:** `runtime/src/select_mod.c`

**Implemented Functions:**
- **FdSet:**
  - `vp_select_fdset_create/free` - Create fd set
  - `vp_select_fdset_add/remove` - Add/remove fd
  - `vp_select_fdset_contains` - Check if fd in set
- **Select:**
  - `vp_select_select(read, write, error, timeout)` - Multiplexed I/O
  - `vp_select_result_free` - Free result
- **Poll (Linux):**
  - `vp_poll_poll(fds, timeout)` - Poll fds
- **Epoll (Linux):**
  - `vp_epoll_create/free` - Create epoll instance
  - `vp_epoll_ctl(ep, op, fd, events)` - Control epoll
  - `vp_epoll_wait(ep, timeout)` - Wait for events
- **Convenience:**
  - `vp_select_can_read/write(fd, timeout)` - Check readiness

**Constants:**
- `EPOLLIN`, `EPOLLOUT`, `EPOLLERR`, `EPOLLHUP`, `EPOLLET`
- `EPOLL_CTL_ADD`, `EPOLL_CTL_MOD`, `EPOLL_CTL_DEL`

**Viper API:**
```viper
import select
readable, _, _ = select.select([sock], [], [], 1.0)
```

---

## Phase 4: Utilities & Security ✅ RUNTIME COMPLETE

### hashlib Module
**Files:** `runtime/src/hashlib.c`

**Implemented Functions:**
- **SHA-256:**
  - `vp_hash_sha256(data, len)` - Compute SHA-256 hash
  - Full implementation (no external dependencies)
- **MD5:**
  - `vp_hash_md5(data, len)` - Compute MD5 hash
- **SHA-512:**
  - `vp_hash_sha512(data, len)` - Compute SHA-512 hash
- **Hash Objects:**
  - `vp_hashlib_new(algo)` - Create hash object
  - `vp_hashlib_update(hash, data, len)` - Update hash
  - `vp_hashlib_digest/hash(hash)` - Get digest

**Constants:**
- Block sizes: MD5=64, SHA256=64, SHA512=128
- Digest sizes: MD5=16, SHA256=32, SHA512=64

**Viper API:**
```viper
import hashlib
h = hashlib.new("sha256")
h.update(b"hello")
print(h.hexdigest())
```

---

### decimal Module
**Files:** `runtime/src/decimal_mod.c`

**Implemented Functions:**
- **Creation:**
  - `vp_decimal_create()` - Create zero decimal
  - `vp_decimal_from_str(str)` - Parse from string
  - `vp_decimal_from_i64(n)` - Create from integer
  - `vp_decimal_from_f64(n)` - Create from float
- **Conversion:**
  - `vp_decimal_to_str(d)` - Convert to string
  - `vp_decimal_to_i64(d)` - Convert to integer
  - `vp_decimal_to_f64(d)` - Convert to float
- **Arithmetic:**
  - `vp_decimal_add/sub/mul/div(a, b)` - Basic operations
  - `vp_decimal_neg/abs(d)` - Unary operations
- **Comparison:**
  - `vp_decimal_cmp/eq/lt/le/gt/ge(a, b)` - Comparisons
- **Rounding:**
  - `vp_decimal_quantize/round(d, places)` - Rounding
  - `vp_decimal_floor/ceil(d)` - Floor/ceiling
- **Properties:**
  - `vp_decimal_get_sign/scale(d)` - Get properties
  - `vp_decimal_is_zero/nan/infinite(d)` - Tests

**Implementation:**
- 128-bit fixed-point representation
- Up to 34 decimal digits precision
- Scale 0-28

**Viper API:**
```viper
from decimal import Decimal
d = Decimal.from_str("3.14159")
```

---

### logging Module
**Files:** `runtime/src/logging.c`

**Implemented Functions:**
- **Logger Creation:**
  - `vp_logging_create_logger(name, level)` - Create logger
  - `vp_logging_get_logger(name)` - Get/create logger
- **Logging:**
  - `vp_logging_debug/info/warning/error/critical(logger, msg)` - Log messages
  - `vp_logging_*_f(logger, format, ...)` - Printf-style logging
  - `vp_logging_exception(logger, msg)` - Log exception
- **Configuration:**
  - `vp_logging_set_level(logger, level)` - Set log level
  - `vp_logging_basic_config(level, format, stream)` - Basic config
  - `vp_logging_add_handler(logger, stream)` - Add output handler
  - `vp_logging_set_format(logger, format)` - Set format string
- **Utilities:**
  - `vp_logging_enabled_for(logger, level)` - Check if enabled
  - `vp_logging_cleanup()` - Cleanup loggers

**Levels:**
- DEBUG (0), INFO (1), WARNING (2), ERROR (3), CRITICAL (4), NOTSET (5)

**Features:**
- Thread-safe (pthread mutex)
- Timestamp formatting
- Multiple output streams

**Viper API:**
```viper
import logging
logging.basic_config(logging.INFO)
logger = logging.get_logger("myapp")
logger.info("Starting...")
```

---

### pathlib Module (Pure Viper)
**Files:** `std/core/pathlib.vp`

**Implemented Path Class:**
- **Properties:** `anchor`, `drive`, `root`, `name`, `suffix`, `stem`, `parent`, `parents`, `parts`
- **Operators:** `/` (join), `==`, `str()`, `repr()`
- **Tests:** `is_absolute`, `is_relative_to`, `exists`, `is_file`, `is_dir`
- **Operations:** `resolve`, `absolute`, `relative_to`, `joinpath`
- **Directory:** `mkdir`, `rmdir`, `iterdir`, `glob`, `rglob`
- **File:** `read_text`, `write_text`, `touch`, `unlink`, `rename`, `copy`, `move`
- **Info:** `stat`, `samefile`, `match`, `with_name`, `with_suffix`, `as_uri`
- **Class Methods:** `cwd()`, `home()`, `absolute_root()`

**Viper API:**
```viper
from pathlib import Path
p = Path("/home/user") / "file.txt"
if p.exists():
    content = p.read_text()
```

---

### argparse Module (Pure Viper)
**Files:** `std/core/argparse.vp`

**Implemented Classes:**
- **ArgumentParser:**
  - `add_argument(name, short, type, default, action, help)` - Add argument
  - `parse_args(args)` - Parse command line
  - `print_help()` - Print help message
  - `error(message)` - Print error and exit
  - `add_argument_group(title)` - Add argument group
  - `add_mutually_exclusive_group()` - Add exclusive group
- **Namespace:**
  - Object for holding parsed arguments
  - `vars()` - Return as dictionary
- **Argument:**
  - Name, short name, type, default, action, help
  - Positional and optional support

**Actions:**
- STORE, STORE_TRUE, STORE_FALSE, APPEND, COUNT, HELP, VERSION

**Type Converters:**
- `int`, `float`, `str`, `boolean`, `positive_int`, `non_negative_int`

**Viper API:**
```viper
from argparse import ArgumentParser
parser = ArgumentParser(description="My tool")
parser.add_argument("--verbose", "-v", action="store_true")
parser.add_argument("input", help="Input file")
args = parser.parse_args()
```

---

## Remaining Work

### JIT Stubs (Phases 2-4)
The following modules have C runtime implementations but need JIT stub registrations:

- **Phase 2:** math, json, collections, re, random
- **Phase 3:** socket, asyncio, http, select
- **Phase 4:** hashlib, decimal, logging

### Viper Wrappers (Phases 3-4)
The following modules need Viper stdlib wrapper files:

- **Phase 3:** socket.vp, asyncio.vp, http.vp, select.vp
- **Phase 4:** hashlib.vp, decimal.vp, logging.vp

### Integration Tests
Create test programs for each phase:
- `tests/test_stdlib_phase1.vp`
- `tests/test_stdlib_phase2.vp`
- `tests/test_stdlib_phase3.vp`
- `tests/test_stdlib_phase4.vp`

---

## Build Instructions

```bash
# Build runtime library
cd runtime && make release

# Build compiler
cd .. && cargo build

# Run tests (when implemented)
cargo test
./target/debug/viper tests/test_stdlib_phase1.vp
```

---

## Notes

1. **Memory Management:** All C runtime functions use Viper's ARC (Automatic Reference Counting) system via `vp_arc_alloc()` and `vp_arc_release()`.

2. **Thread Safety:** The logging module uses pthread mutexes for thread-safe operations. Socket operations are thread-safe at the OS level.

3. **Platform Support:** 
   - Primary: Linux (tested)
   - Secondary: macOS, FreeBSD (should work with minor adjustments)
   - Windows: Not tested, may require Win32 socket/networking adaptations

4. **Dependencies:**
   - Standard C library
   - POSIX: `<regex.h>`, `<sys/socket.h>`, `<sys/select.h>`, `<pthread.h>`
   - Math: `<math.h>`
   - No third-party dependencies

5. **Known Limitations:**
   - HTTP client is simplified (placeholder implementation)
   - Asyncio adapter needs full coroutine integration
   - Decimal uses simplified 64-bit coefficient (not full 128-bit)
   - JSON parser returns simplified structures
