
I should be able to understand after this lab RFC 1951 (the DEFLATE spec).

# Lab: Mini Deflate

Goal:

```text
compress input.bin output.mdf
decompress output.mdf restored.bin

assert(restored == input)
```

It doesn't have to beat gzip. It just has to work correctly.

---

# Knowledge you'll need

## 1. Binary files

You are no longer working with text.

Everything is just bytes.

```text
0A FF 34 82 ...
```

Know:

* `Vec<u8>`
* read/write files
* byte order
* file headers

---

## 2. Bit manipulation

Need:

* masks
* shifts
* set/clear bits
* extract bits
* pack fields

Example:

```text
101 | 01 | 1110
```

stored as

```text
10101110
```

---

## 3. Bit streams ⭐

Probably the biggest new concept.

Instead of writing bytes:

```text
10101010
11110000
```

you write

```text
1
01
111
0
10
...
```

continuously.

Need:

```rust
BitWriter
BitReader
```

This teaches buffering.

---

## 4. LZ77

This is the first real compression stage.

Instead of

```text
banana banana banana
```

store

```text
banana
(distance=7,length=6)
(distance=7,length=6)
```

Concepts:

* sliding window
* search previous bytes
* longest match
* literals
* back references

Need to know:

```text
distance
length
next byte
```

---

## 5. Huffman

Huffman coding gives **frequent symbols shorter bit codes** and rare symbols longer codes.

Example frequencies:

```text
A: 5
B: 2
C: 1
D: 1
```

Build process:

```text
1. Put every symbol in a min-priority queue.
2. Remove the two least frequent nodes.
3. Combine them into a parent whose frequency is their sum.
4. Put the parent back.
5. Repeat until one tree remains.
```

Possible tree:

```text
        9
       / \
     A:5  4
         / \
       B:2  2
           / \
         C:1 D:1
```

Assign:

```text
left  = 0
right = 1
```

Codes become:

```text
A = 0
B = 10
C = 110
D = 111
```

So:

```text
A B A C
```

becomes:

```text
0 10 0 110
```

or:

```text
0100110
```

## Why decoding works

Huffman codes are **prefix-free**: no code is the beginning of another code.

Decode by walking the tree:

```text
0 → left
1 → right
```

When you reach a leaf, output that symbol and restart from the root.

## What you implement

```rust
struct Node {
    frequency: usize,
    symbol: Option<u16>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}
```

Main pieces:

```text
count_frequencies()
build_tree()
generate_codes()
encode()
decode()
```

For Mini DEFLATE, the symbols are not only characters. They include:

```text
literal byte values
length values
distance values
end-of-block marker
```

Huffman does **not find repetition**. LZ77 does that first; Huffman then stores the resulting tokens with fewer bits.

Need to implement:

* frequency table

```text
A : 50

B : 15

C : 2
```

↓

priority queue

↓

tree

↓

codes

↓

bit stream

You'll finally understand why variable-length codes need a bit writer.

---

## 6. Tokens

LZ77 doesn't output bytes.

It outputs something like

```text
Literal('A')

Literal('B')

Literal('C')

Match {
    distance: 3,
    length: 8
}
```

Then Huffman compresses those tokens.

---

## 7. Custom binary format

Need your own format.

Example

```text
MAGIC

VERSION

FLAGS

HUFFMAN TABLE

COMPRESSED DATA
```

Parser

Serializer

Validation

---

## 8. Error handling

Reject

* bad magic
* truncated file
* invalid distance
* invalid Huffman tree
* unexpected EOF

---

## 9. Performance

Not optimization.

Just measurements.

Show

```text
Original:
5.2 MB

Compressed:
2.4 MB

Ratio:
46%

Compression:
18 MB/s

Decompression:
71 MB/s
```

---

# Requirements

## Commands

```text
mini-deflate

compress INPUT OUTPUT

decompress INPUT OUTPUT

verify ORIGINAL DECOMPRESSED

inspect FILE

benchmark FILE
```

---

## Compress

Requirements

* read binary file
* LZ77 encode
* Huffman encode
* write custom file

---

## Decompress

Requirements

* read file
* validate header
* decode Huffman
* reconstruct LZ77
* restore original bytes

---

## Verify

```text
Original:
SHA256

Decoded:
SHA256

✓ identical
```

or compare byte-by-byte.

---

## Inspect

Print

```text
Version

Original size

Compressed size

Compression ratio

Literal count

Match count

Average match length

Longest match
```

This becomes debugging tool.

---

## Benchmark

Measure

```text
Compression time

Decompression time

Throughput

Ratio
```

---

# Implementation order

```text
01 Read/write binary files

02 BitWriter

03 BitReader

04 LZ77 encoder

05 LZ77 decoder

06 Huffman encoder

07 Huffman decoder

08 File format

09 CLI

10 Benchmark

11 Optimizations
```

Don't write a single line of Huffman until the LZ77 encoder/decoder works perfectly.

---

# Stretch goals
(Optional)
* CRC32 checksum
* Multi-file archive (`.zip`-like)
* Streaming compression (don't load the whole file)
* Parallel compression
* Dynamic vs static Huffman
* Compare against `gzip`
* Support stdin/stdout

---

