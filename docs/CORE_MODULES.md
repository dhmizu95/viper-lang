Power 20 Standard Library – Implementation Plan
Implement 20 Python-compatible standard library modules for Viper. Each module follows the same three-layer pattern used throughout the codebase:

C Runtime (runtime/src/<module>.c) — exposes vp_<module>_* functions, linked into libviper.a
Viper Stdlib (std/<module>.vp) — extern declarations + thin domain-logic wrappers
Compiler wiring — the existing module-resolver in src/jit_stubs/ or src/semantic/ must recognise import <module> and map symbols
Proposed Changes
Phase 1 – Runtime Foundation
[MODIFY] 
runtime.c
Extend vp_builtin_range_list with the optional start/step overloads (range(start,stop,step))
Add vp_builtin_input() → reads a line from stdin, returns char*
Add vp_builtin_enumerate(ViperList*) → returns list of [i64, value] tuples
zip, map, filter, sorted, reversed — list-returning helpers
[NEW] 
std/builtins.vp
Extern bindings for all new runtime helpers.

[NEW] 
runtime/src/sys.c
vp_sys_argv(), vp_sys_exit(int), vp_sys_getpid(), vp_sys_version(), vp_sys_platform()

[NEW] 
std/sys.vp
Exposes argv: [str], exit(code), version: str, platform: str

[NEW] 
runtime/src/os.c
POSIX wrappers: vp_os_getcwd(), vp_os_listdir(path), vp_os_path_join(a,b), vp_os_getenv(name), vp_os_mkdir(path), vp_os_remove(path), vp_os_getpid()

[NEW] 
std/os.vp
Thin Viper wrappers + os.path as a namespace object.

[NEW] 
runtime/src/gc.c
vp_gc_collect(), vp_gc_disable(), vp_gc_enable(), vp_gc_get_count() — hooks into the existing ARC engine in runtime/src/memory/arc.c.

[NEW] 
std/gc.vp
[NEW] 
runtime/src/time_mod.c
vp_time_time() (wall clock f64), vp_time_monotonic() (CLOCK_MONOTONIC), vp_time_sleep(f64), vp_time_perf_counter()

[NEW] 
std/time.vp
Phase 2 – Data & Performance
[NEW] 
runtime/src/math_mod.c
All <math.h> intrinsics: sqrt, cbrt, floor, ceil, trunc, round, log, log2, log10, exp, sin, cos, tan, asin, acos, atan, atan2, fabs, fmod, pow, hypot. Constants: pi, e, tau, inf, nan as f64 globals.

[NEW] 
std/math.vp
Extern all; add domain-check wrappers (sqrt raises for negative).

[NEW] 
runtime/src/json.c
Recursive-descent JSON parser: vp_json_loads(str) -> ViperDict*, vp_json_dumps(ViperDict*) -> char*. Uses only <string.h> — no third-party deps.

[NEW] 
std/json.vp
loads(s: str), dumps(obj), JSONDecodeError

[NEW] 
runtime/src/collections.c
ViperDeque (doubly-linked, O(1) head/tail), ViperCounter (dict-backed), ViperOrderedDict (hash-map + insertion-order list).

[NEW] 
std/collections.vp
deque, Counter, OrderedDict, defaultdict, namedtuple

[NEW] 
runtime/src/re_mod.c
POSIX <regex.h> wrappers: vp_re_match, vp_re_search, vp_re_findall, vp_re_sub, vp_re_split. Compiled patterns are cached via a simple LRU (16-slot).

[NEW] 
std/re.vp
compile(pattern, flags) -> Pattern, match/search/findall/sub/split, Pattern.match() etc.

[NEW] 
runtime/src/random_mod.c
PCG64 PRNG: vp_random_random() → f64 in [0,1), vp_random_randint(a,b), vp_random_seed(n), vp_random_choice(ViperList*), vp_random_shuffle(ViperList*). Falls back to getrandom() for secure seeding.

[NEW] 
std/random.vp
Phase 3 – Networking & Concurrency
[NEW] 
runtime/src/socket_mod.c
POSIX: vp_socket_create, vp_socket_bind, vp_socket_listen, vp_socket_accept, vp_socket_connect, vp_socket_send, vp_socket_recv, vp_socket_close.

[NEW] 
std/socket.vp
Socket class with AF_INET/AF_INET6/SOCK_STREAM/SOCK_DGRAM constants.

[NEW] 
runtime/src/asyncio_mod.c
Thin adapter over existing event_loop_epoll.c: vp_asyncio_run(fn), vp_asyncio_sleep(f64), vp_asyncio_gather(fns[], n).

[NEW] 
std/asyncio.vp
run(coro), sleep(secs), gather(*coros), create_task(coro), Event, Lock, Queue

[NEW] 
runtime/src/http_mod.c
HTTP/1.1 client over raw sockets: vp_http_get(url), vp_http_post(url, body). Returns ViperDict{status, headers, body}. Minimal server: vp_http_serve(port, handler_fn_ptr).

[NEW] 
std/http.vp
get(url) -> Response, post(url, data) -> Response, Response{status_code, text, json()}, serve(port, handler)

[NEW] 
std/task.vp
Viper-exclusive. Extern bindings directly into existing thread_pool.c and scheduler.c symbols: spawn(fn, *args), yield_now(), current_task_id(), set_concurrency(n).

[NEW] 
runtime/src/select_mod.c
vp_select_select(read_fds, write_fds, timeout) — wraps epoll_wait on Linux, select() fallback.

[NEW] 
std/select.vp
select(rlist, wlist, xlist, timeout) -> (r, w, x)

Phase 4 – Utilities & Security
[NEW] 
std/pathlib.vp
Path class implemented purely in Viper using os module: __div__ operator for /-joining, .stat(), .read_text(), .write_text(), .exists(), .iterdir(), .mkdir(), .unlink().

[NEW] 
runtime/src/hashlib.c
Self-contained SHA-256, MD5, SHA-512 implementations (no OpenSSL dep). vp_hash_sha256(data,len), vp_hash_md5(data,len), vp_hash_sha512(data,len) — return hex strings.

[NEW] 
std/hashlib.vp
new(algo) -> Hash, Hash.update(data), Hash.hexdigest(), Hash.digest()

[MODIFY] 
std/decimal.vp
Replace the skeleton with a real Decimal class backed by a new C module.

[NEW] 
runtime/src/decimal_mod.c
128-bit fixed-point ops: vp_decimal_add, vp_decimal_sub, vp_decimal_mul, vp_decimal_div, vp_decimal_from_str, vp_decimal_to_str.

[NEW] 
runtime/src/logging.c
Thread-safe logger using a mutex-protected file descriptor: vp_log_debug/info/warning/error/critical(name, msg). Levels controlled by a global threshold.

[NEW] 
std/logging.vp
getLogger(name) -> Logger, Logger.{debug,info,warning,error,critical}(msg), basicConfig(level, format)

[NEW] 
std/argparse.vp
Pure Viper: ArgumentParser class that reads sys.argv, supports add_argument(name, type, default, help), parse_args() -> Namespace, --help auto-generation.

Compiler Wiring (applies to all modules)
[MODIFY] runtime/Makefile
Add explicit compile rules and link targets for all new .c files: sys.o, os.o, gc.o, time_mod.o, math_mod.o, json.o, collections.o, re_mod.o, random_mod.o, socket_mod.o, asyncio_mod.o, http_mod.o, select_mod.o, hashlib.o, decimal_mod.o, logging.o.

[MODIFY] src/jit_stubs/ or src/semantic/
Register each new module name so import sys, import os, etc. resolve at compile time. Follow the same pattern as the existing typing module registration in src/typing_module.rs.

Verification Plan
Automated Tests
bash
# Build the runtime first
cd runtime && make release
# Run all Rust unit + integration tests
cargo test --workspace
# Test each phase with a .vp integration file
./target/debug/viper tests/viper_programs/test_stdlib_phase1.vp
./target/debug/viper tests/viper_programs/test_stdlib_phase2.vp
./target/debug/viper tests/viper_programs/test_stdlib_phase3.vp
./target/debug/viper tests/viper_programs/test_stdlib_phase4.vp
I will write four new integration test programs in tests/viper_programs/:

test_stdlib_phase1.vp — imports and exercises builtins, sys, os, gc, time
test_stdlib_phase2.vp — math, json, collections, re, random
test_stdlib_phase3.vp — socket (loopback), asyncio, task, select
test_stdlib_phase4.vp — pathlib, hashlib, decimal, logging, argparse
Each program uses assert to validate results and prints OK on success.