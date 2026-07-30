# Lab - Linux Input Method (IME)

## Goal

Build a minimal Linux input method that works in normal applications.

Supported rules:

```text
aa -> â
aw -> ă
dd -> đ
```

No dictionaries. No prediction. No full Vietnamese support.

---

# What to learn

## 1. Linux desktop input architecture

Understand:

```text
Keyboard
    ↓
IBus
    ↓
Your IME
    ↓
Application
```

Learn:

* IME
* preedit
* commit
* input context

---

## 2. D-Bus (high level)

Understand:

* What IPC is
* Why processes can't call each other directly
* How IBus communicates with applications

No need to implement D-Bus.

---

## 3. IBus API

Learn:

* Engine lifecycle
* Key event callback
* Update preedit
* Commit text
* Reset composition

---

## 4. Rust/C++ interop

Architecture:

```text
IBus (C++)
      ↓
Rust Telex engine
```

Rust owns:

* composition state
* transformation rules

C++ owns:

* IBus callbacks
* preedit
* commit

---

# Milestones

## Milestone 1

Create an IBus engine.

Success:

```text
Engine appears in IBus settings.
```

---

## Milestone 2

Receive key events.

Success:

```text
Typing logs every key.
```

---

## Milestone 3

Commit fixed text.

Example:

```text
F8

↓

Hello
```

Learn:

* commit text

---

## Milestone 4

Implement preedit.

```text
a

↓

(preedit) a

↓

aa

↓

(preedit) â

↓

Space

↓

(commit) â
```

Learn:

* composition

---

## Milestone 5

Move Telex logic into Rust.

```text
IBus
    ↓
Rust engine
    ↓
Action
    ↓
IBus updates UI
```

---

## Milestone 6

Implement:

```text
aa -> â
aw -> ă
dd -> đ
Backspace
Escape
Space commits
```

---

# Reading

1. APUE

   * Daemons (refresh)
   * Process lifecycle

2. D-Bus tutorial

   * Basic concepts only

3. IBus documentation

   * Engine
   * Preedit
   * Commit

4. Existing IME source

   * Follow one simple engine to understand the control flow

---

# Out of scope

* Full Vietnamese grammar
* Tone placement
* Candidate window
* Prediction
* Dictionaries
* Wayland/X11 internals
* Implementing D-Bus
* Raw keyboard hooks

---

## Final architecture

```text
Keyboard
    ↓
IBus
    ↓
C/C++ Engine
    ↓
Rust Telex Library
    ↓
Action (Update / Commit / Pass)
    ↓
IBus
    ↓
Application
```

