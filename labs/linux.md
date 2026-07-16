# Linux Reference

Commands I've actually used while building this repo.

---

## Binary

### file

```bash
file program
```

Identify file type (ELF, executable, text, etc.)

---

### size

```bash
size program
```

Show text/data/bss size.

---

### strings

```bash
strings program
```

Show printable strings inside a binary.

---

### nm

```bash
nm program
```

List symbols.

---

### readelf

```bash
readelf -h program
readelf -S program
readelf -s program
```

Inspect ELF.

---

### objdump

```bash
objdump -d program
objdump -S program
```

Disassemble executable.

---

## Process

### ps

```bash
ps aux
```

List running processes.

---

### pgrep

```bash
pgrep program
```

Find process ID.

---

### top

```bash
top
```

Live process monitor.

---

### kill

```bash
kill PID
kill -9 PID
```

Send signal to process.

---

## Files

### lsof

```bash
lsof -p PID
lsof -i :7878
```

List open files and sockets.

---

## Networking

### ss

```bash
ss -ltnp
```

Inspect sockets.

---

### curl

```bash
curl -i localhost:7878
curl -v localhost:7878
```

Send HTTP requests.

---

### nc

```bash
nc localhost 7878
```

Open raw TCP connection.

---

## Debugging

### strace

```bash
strace program
strace -e trace=network program
```

Trace system calls.

---

### gdb

```bash
gdb program
```

Native debugger.

---

## Memory

### pmap

```bash
pmap PID
```

View process memory map.

---

## Performance

### time

```bash
time program
```

Measure execution time.

---

### perf

```bash
perf stat program
perf record program
perf report
```

Performance profiling.

---

## Bytes

### hexdump

```bash
hexdump -C file
```

View raw bytes.

---

### xxd

```bash
xxd file
```

Hex dump.

---

## Search

### rg

```bash
rg pattern
```

Fast grep.

---

### find

```bash
find . -name "*.rs"
```

Find files.

---

### tree

```bash
tree
```

Directory tree.
