# Viper Standard Library Reference

Complete reference for all standard library modules in Viper.

## Table of Contents

1. [Core Modules](#core-modules)
2. [Data Types](#data-types)
3. [Networking](#networking)
4. [Utilities](#utilities)
5. [Development Tools](#development-tools)

---

## Core Modules

### os - Operating System Interface

```python
import os

# Directory operations
os.getcwd()                    # Get current working directory
os.chdir("/path/to/dir")      # Change directory
os.listdir("/path")           # List directory contents
os.mkdir("newdir")            # Create directory
os.makedirs("a/b/c")          # Create directories recursively
os.rmdir("emptydir")          # Remove empty directory
os.remove("file.txt")         # Remove file
os.rename("old.txt", "new.txt")  # Rename file

# File operations
os.exists("file.txt")         # Check if path exists
os.isfile("file.txt")         # Check if regular file
os.isdir("dir")               # Check if directory
os.getsize("file.txt")        # Get file size in bytes
os.getmtime("file.txt")       # Get modification time
os.getatime("file.txt")       # Get access time

# Path operations
os.path.join("a", "b", "c")   # Join path components
os.path.abspath("file.txt")   # Get absolute path
os.path.basename("/a/b.txt")  # Get basename: "b.txt"
os.path.dirname("/a/b.txt")    # Get directory: "/a"
os.path.splitext("file.txt")  # Split extension: ("file", ".txt")
os.path.isabs("/path")        # Check if absolute
os.path.normpath("a/../b")    # Normalize path

# Environment variables
os.getenv("HOME")             # Get environment variable
os.setenv("KEY", "value")     # Set environment variable
os.unsetenv("KEY")            # Remove environment variable
os.environ()                  # Get all env vars as dict

# Process information
os.getpid()                   # Get process ID
os.getppid()                  # Get parent process ID
os.getuid()                   # Get user ID
os.getgid()                  # Get group ID

# System
os.system("command")          # Execute shell command
os.get_home()                 # Get home directory
os.expanduser("~/file.txt")   # Expand ~ to home

# File descriptors
fd = os.open("file.txt", os.O_RDONLY)
data = os.read(fd, 100)
os.write(fd, "hello")
os.close(fd)
```

### sys - System Parameters

```python
import sys

sys.exit(0)                       # Exit program
sys.getpid()                      # Get process ID
sys.get_platform()                 # Get platform: "linux", "darwin", "windows"
sys.get_version()                  # Get Viper version
sys.get_sysname()                  # Get system name
sys.get_machine()                  # Get machine architecture
sys.getargv()                      # Get command-line arguments
sys.getrecursionlimit()            # Get recursion limit
sys.setrecursionlimit(10000)       # Set recursion limit
sys.getsizeof(obj)                 # Get object size in bytes
sys.getrefcount(obj)               # Get reference count
```

### time - Time and Date

```python
import time

# Time functions
time.time()                       # Unix timestamp (seconds since epoch)
time.monotonic()                   # Monotonic clock
time.perf_counter()                # High-resolution performance counter

# Sleep
time.sleep(1.0)                    # Sleep for 1 second
time.sleep_ms(500)                 # Sleep for 500 milliseconds
time.sleep_us(1000)                # Sleep for 1000 microseconds

# Time conversion
time.localtime()                   # Local time as struct
time.gmtime()                      # UTC time as struct
time.mktime(t)                     # Convert struct to timestamp

# Formatting
time.strftime("%Y-%m-%d %H:%M:%S")  # Format time
time.strptime("2024-01-01", "%Y-%m-%d")  # Parse time
time.asctime()                     # Format as string
time.ctime()                       # Current time as string
time.isoformat()                   # ISO 8601 format

# Timezone
time.timezone()                    # Timezone offset in seconds
time.tzname()                      # Timezone names
time.daylight()                    # Is DST in effect
time.tzset()                       # Initialize timezone

# Date utilities
time.days_in_month(2024, 2)        # Days in month
time.is_leap_year(2024)            # Is leap year
time.day_of_week(2024, 1, 1)       # Day of week (0=Monday)
time.day_of_year(2024, 1, 1)      # Day of year (1-366)
time.make_time(2024, 1, 1, 12, 0, 0)  # Make timestamp
```

### math - Mathematical Functions

```python
import math

# Basic functions
math.ceil(3.2)                     # 4.0
math.floor(3.8)                    # 3.0
math.trunc(3.8)                    # 3
math.fabs(-3.5)                    # 3.5
math.fmod(10, 3)                   # 1.0
math.remainder(10, 3)              # 1.0
math.copysign(5, -3)               # -5.0

# Exponential and logarithmic
math.exp(1)                        # e^1 ≈ 2.718
math.expm1(1)                      # e^1 - 1
math.log(10)                       # Natural log
math.log(10, 2)                   # Log base 2
math.log1p(1)                      # ln(1 + 1)
math.log2(8)                       # 3.0
math.log10(100)                    # 2.0

# Power and roots
math.pow(2, 3)                     # 8.0
math.sqrt(16)                      # 4.0
math.cbrt(8)                       # 2.0
math.isqrt(10)                     # 3 (integer sqrt)

# Trigonometric
math.sin(0)                        # 0.0
math.cos(0)                        # 1.0
math.tan(0)                        # 0.0
math.asin(0)                       # 0.0
math.acos(1)                       # 0.0
math.atan(1)                       # 0.785...
math.atan2(y, x)                   # Arc tangent
math.hypot(3, 4)                   # 5.0 (Euclidean distance)

# Hyperbolic
math.sinh(0)                       # 0.0
math.cosh(0)                       # 1.0
math.tanh(0)                       # 0.0
math.asinh(0)                      # 0.0
math.acosh(1)                      # 0.0
math.atanh(0)                      # 0.0

# Angular conversion
math.degrees(3.14159)               # Convert radians to degrees
math.radians(180)                   # Convert degrees to radians

# Special functions
math.erf(0)                        # Error function
math.erfc(0)                       # Complementary error
math.gamma(5)                      # Gamma function
math.lgamma(5)                     # Log gamma

# Integer math
math.gcd(48, 18)                   # 6 (greatest common divisor)
math.lcm(4, 6)                     # 12 (least common multiple)
math.factorial(5)                  # 120
math.comb(5, 2)                    # 10 (combinations)
math.perm(5, 2)                    # 20 (permutations)

# Float classification
math.isfinite(1.0)                 # True
math.isinf(float('inf'))           # True
math.isnan(float('nan'))           # True
math.isnormal(1.0)                 # True
math.signbit(-1.0)                 # True

# Reduction
math.fsum([0.1, 0.1, 0.1])         # Accurate sum
math.prod([1, 2, 3, 4])            # 24
math.mean([1, 2, 3, 4])            # 2.5
math.fmin(1, 2)                    # 1.0
math.fmax(1, 2)                    # 2.0

# Vector math
math.dist([0, 0], [3, 4])          # 5.0 (distance)
math.clamp(5, 0, 10)               # 5 (constrained)
math.lerp(0, 10, 0.5)              # 5.0 (linear interp)
math.smoothstep(0, 1, 0.5)         # 0.5 (Hermite)

# Constants
math.pi                            # 3.14159...
math.e                             # 2.71828...
math.tau                            # 6.28318...
math.inf                            # Infinity
math.nan                            # Not a number
```

---

## Data Types

### json - JSON Serialization

```python
import json

# Parsing
data = json.loads('{"name": "Alice", "age": 30}')
data = json.load(file_pointer)
data = json.load_file("data.json")

# Serialization
json_str = json.dumps({"name": "Alice", "age": 30})
json_str = json.dumps(data, indent=4, sort_keys=True)
json.dump(data, file_pointer)
json.dump_file(data, "output.json")

# Validation
is_valid = json.is_valid_json('{"key": "value"}')

# Conversion
json_str = json.to_json(obj)           # Object to JSON
obj = json.from_json(json_str)         # JSON to object
```

### collections - Collection Utilities

```python
import collections

# Named tuples
Point = collections.namedtuple("Point", ["x", "y"])
p = Point(10, 20)
print(p.x)  # 10
print(p.y)  # 20

# Counter (frequency counting)
counts = collections.Counter([1, 2, 2, 3, 3, 3])
print(counts.most_common(2))  # [(3, 3), (2, 2)]

# Default dict
d = collections.defaultdict(int)
d["a"] = d["a"] + 1  # Works without initialization
```

### decimal - Decimal Arithmetic

```python
import decimal

# Create decimal
d = decimal.create_decimal("3.14159")
d = decimal.decimal_zero()
d = decimal.decimal_one()
d = decimal.decimal_pi()      # Pi with precision
d = decimal.decimal_e()       # e with precision

# Operations
result = d1 + d2
result = d1 - d2
result = d1 * d2
result = d1 / d2

# Conversion
f = decimal.decimal_to_float(d)
s = decimal.decimal_to_string(d)
```

### fractions - Fraction Arithmetic

```python
import fractions

# Create fraction
f = fractions.create_fraction(3, 4)    # 3/4
f = fractions.fraction_from_int(5)      # 5/1
f = fractions.fraction_from_float(0.5)   # Approximate

# Operations
result = f1 + f2
result = f1 - f2
result = f1 * f2
result = f1 / f2

# Properties
num = fractions.fraction_numerator(f)   # 3
den = fractions.fraction_denominator(f) # 4

# Conversion
float_val = fractions.fraction_to_float(f)
str_val = fractions.fraction_to_string(f)
```

### hashlib - Cryptographic Hashing

```python
import hashlib

# Create hash objects
h = hashlib.md5()
h = hashlib.sha1()
h = hashlib.sha256()
h = hashlib.sha512()
h = hashlib.new("sha256")

# Update hash with data
h.update("Hello")
h.update(" World")

# Get digest
digest = h.digest()        # Raw bytes
hexdigest = h.hexdigest()  # Hex string

# Convenience functions
result = hashlib.hash_sha256("data")
result = hashlib.hash_md5("data")
result = hashlib.hash_sha512("data")

# HMAC
hmac = hashlib.hmac_new("key", "message", "sha256")
result = hmac.digest()

# Key derivation
derived = hashlib.pbkdf2_hmac("password", "salt", 100000, 32)

# Constant-time comparison
equal = hashlib.compare_digest(a, b)
```

---

## Networking

### socket - Network Sockets

```python
import socket

# Create socket
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

# Connect to server
sock.connect(("example.com", 80))

# Send/Receive
sock.send("GET / HTTP/1.0\r\n\r\n")
data = sock.recv(1024)

# Close
sock.close()

# Server
server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(("0.0.0.0", 8080))
server.listen(5)
client, addr = server.accept()

# Convenience
ip = socket.gethostbyname("example.com")
hostname = socket.gethostname()
conn = socket.create_connection(("example.com", 80))
```

### http - HTTP Client/Server

```python
import http

# Client requests
response = http.get("https://example.com")
response = http.post("https://example.com", data="body")
response = http.put("https://example.com", data="body")
response = http.delete("https://example.com")
response = http.patch("https://example.com", data="body")
response = http.head("https://example.com")
response = http.options("https://example.com")

# Response properties
status = response.status      # 200
body = response.body          # Response body
headers = response.headers   # Dict of headers

# Request with custom headers
response = http.get("https://api.example.com", 
    headers={"Authorization": "Bearer token"})

# Server
server = http.create_server(8080)
# Handle requests in handler function

# URL utilities
encoded = http.urlencode({"key": "value"})
decoded = http.urldecode("key=value")
joined = http.urljoin("https://example.com", "/path")
parsed = http.urlparse("https://user:pass@example.com:8080/path")
```

### ssl - SSL/TLS

```python
import ssl

# Create SSL context
ctx = ssl.create_context(ssl.CERT_NONE)

# Wrap socket
ssl_sock = ssl.wrap_socket(sock, 
    server_side=False,
    do_handshake_on_connect=True)

# Read/Write
data = ssl.read(ssl_sock, 1024)
ssl.write(ssl_sock, "data")

# Get peer certificate
cert = ssl.getpeercert(ssl_sock)

# Cleanup
ssl.close(ssl_sock)
ssl.free_context(ctx)
```

---

## Utilities

### random - Random Numbers

```python
import random

# Random numbers
random.random()                    # Float 0.0 to 1.0
random.randint(1, 100)              # Integer 1 to 100
random.randrange(10)                # Integer 0 to 9
random.randrange(5, 10)             # Integer 5 to 9
random.randrange(0, 10, 2)          # Even numbers 0-8

# Sequences
choice = random.choice([1, 2, 3])   # Random element
shuffled = random.shuffle([1, 2, 3])  # Shuffle in place
sample = random.sample([1, 2, 3, 4, 5], 2)  # Multiple choices

# Distributions
normal = random.gauss(0, 1)         # Gaussian/normal distribution
exp = random.expovariate(1)         # Exponential
uniform = random.uniform(1, 10)     # Uniform distribution
```

### logging - Logging

```python
import logging

# Get logger
logger = logging.get_logger("myapp")
root = logging.get_root_logger()

# Set level
logger.set_level(logging.DEBUG)
logger.set_level(logging.INFO)
logger.set_level(logging.WARNING)
logger.set_level(logging.ERROR)
logger.set_level(logging.CRITICAL)

# Log messages
logger.debug("Debug message")
logger.info("Info message")
logger.warning("Warning message")
logger.error("Error message")
logger.critical("Critical message")

# Convenience functions
logging.debug("Debug")
logging.info("Info")
logging.warning("Warning")
logging.error("Error")
logging.critical("Critical")

# Basic configuration
logging.basic_config(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    filename="app.log"
)
```

### itertools - Iterator Utilities

```python
import itertools

# Count
counter = itertools.count(10)       # Infinite counter
for i in range(5):
    next(counter)                    # 10, 11, 12, 13, 14

# Cycle
cycled = itertools.cycle([1, 2, 3])  # Infinite repetition
for i in range(5):
    next(cycled)                     # 1, 2, 3, 1, 2

# Repeat
repeated = itertools.repeat(5, 3)   # Repeat 5 three times

# Chain
chained = itertools.chain([1, 2], [3, 4])  # 1, 2, 3, 4

# Zip
zipped = itertools.zip_longest([1, 2], [3, 4, 5])  # Pairs

# Accumulate
acc = itertools.accumulate([1, 2, 3, 4])  # 1, 3, 6, 10

# Filter
filtered = itertools.filterfalse(lambda x: x % 2 == 0, [1, 2, 3, 4])  # 1, 3

# GroupBy
grouped = itertools.groupby([1, 1, 2, 2, 2, 3])
```

### re - Regular Expressions

```python
import re

# Match
match = re.match(r"(\w+)@(\w+)\.(\w+)", "user@example.com")
if match:
    full = match.group(0)    # Full match
    user = match.group(1)     # "user"
    domain = match.group(2)  # "example"
    tld = match.group(3)      # "com"

# Search
match = re.search(r"\d+", "abc123def")
if match:
    print(match.group(0))     # "123"

# Find all
matches = re.findall(r"\d+", "123 abc 456")  # ["123", "456"]

# Substitute
result = re.sub(r"\d+", "X", "123 abc 456")  # "X abc X"

# Split
parts = re.split(r"\s+", "hello world")  # ["hello", "world"]

# Compile for efficiency
pattern = re.compile(r"(\w+)@(\w+)\.(\w+)")
match = pattern.match("user@example.com")
```

### asyncio - Asynchronous Programming

```python
import asyncio

# Run async function
asyncio.run(main())

# Create task
task = asyncio.create_task(coro)

# Run multiple concurrently
results = asyncio.gather(coro1, coro2, coro3)

# Wait for tasks
done, pending = asyncio.wait(tasks, timeout=60)

# Wait for single with timeout
result = asyncio.wait_for(coro, timeout=5.0)

# Event loop
loop = asyncio.new_event_loop()
asyncio.set_event_loop(loop)
loop.close()

# Sleep (non-blocking)
asyncio.sleep(1)  # Non-blocking sleep

# Create Future
future = asyncio.Future()
```

---

## Development Tools

### gc - Garbage Collection

```python
import gc

gc.enable()           # Enable automatic GC
gc.disable()          # Disable automatic GC
gc.isenabled()        # Check if enabled

gc.collect()          # Force collection
gc.set_threshold(10000)  # Set threshold
gc.get_threshold()    # Get threshold
gc.get_count()        # Get collection count

gc.get_memory_usage()     # Current memory
gc.get_total_freed()      # Total freed
gc.get_object_count()     # Tracked objects
gc.get_pending_count()    # Pending finalization
gc.break_cycles()         # Force cycle cleanup
gc.reset_stats()          # Reset statistics

gc.get_stats()        # Get statistics string
gc.print_stats()      # Print to stdout
gc.set_debug(True)   # Enable debug mode
```

### sys - Development Utilities

```python
import sys

# Exit
sys.exit(0)           # Exit with code
sys.exit("error")     # Exit with message

# Recursion
sys.getrecursionlimit()   # Get limit (usually 1000)
sys.setrecursionlimit(5000)  # Increase limit

# Object info
sys.getsizeof(obj)    # Size in bytes
sys.getrefcount(obj)  # Reference count
```

---

## File I/O

### io - Input/Output

```python
import io

# StringIO (in-memory text)
sio = io.StringIO()
sio.write("hello")
sio.write("world")
content = sio.getvalue()
sio.close()

# BytesIO (in-memory binary)
bio = io.BytesIO()
bio.write(b"hello")
bio.write(b"world")
data = bio.getvalue()
bio.close()

# File-like objects
f = io.open("file.txt", "r")
content = f.read()
f.close()
```

### fileinput - Command Line File Input

```python
import fileinput

# Iterate over lines
for line in fileinput.input("file.txt"):
    print(line.rstrip())

# With multiple files
for line in fileinput.input(["a.txt", "b.txt"]):
    print(line)

# In-place editing
# fileinput.input("file.txt", inplace=True)

# File info
filename = fileinput.filename()   # Current filename
lineno = fileinput.lineno()        # Global line number
filelineno = fileinput.filelineno() # Line in current file
```

### tempfile - Temporary Files

```python
import tempfile

# Temporary file
tf = tempfile.TemporaryFile()
tf.write("data")
tf.seek(0)
content = tf.read()
tf.close()

# Named temporary file
ntf = tempfile.NamedTemporaryFile(delete=False)
ntf.close()

# Temporary directory
tdir = tempfile.TemporaryDirectory()
path = tdir.name
tdir.cleanup()

# Utilities
tempdir = tempfile.gettempdir()
tempprefix = tempfile.gettempprefix()
path = tempfile.mkdtemp()
fd, path = tempfile.mkstemp()
```

### shutil - File Operations

```python
import shutil

shutil.copy("src.txt", "dst.txt")   # Copy file
shutil.copy2("src.txt", "dst.txt")  # Copy with metadata
shutil.move("src.txt", "dst.txt")   # Move file
shutil.delete_file("file.txt")      # Delete file
shutil.rmdir("dir")                 # Remove directory
shutil.makedirs("a/b/c", exist_ok=True)  # Make dirs
shutil.which("python")              # Find executable
shutil.get_terminal_size()          # Terminal size
shutil.disk_usage("/")              # Disk usage
shutil.read_file("file.txt")        # Read entire file
shutil.write_file("file.txt", "content")  # Write file
```

---

## Module Loading

### import - Module System

```python
# Import module
import os
import sys as system

# Import specific items
from os import getcwd, chdir
from os import getcwd as cwd

# Import all (not recommended)
from os import *

# Reload module
import importlib
importlib.reload(module)

# Check module
import os
if hasattr(os, "getcwd"):
    # Has getcwd
    pass

# Get module info
module = importlib.import_module("os")
```

---

## See Also

- [LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md) - Language syntax and features
- [README.md](../README.md) - Project overview
- [INSTALLATION.md](../INSTALLATION.md) - Installation instructions
