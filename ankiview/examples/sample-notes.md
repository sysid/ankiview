# Sample Anki Notes in Markdown

This is an example markdown file showing how to write Anki flashcards.

---
Deck: Programming
Tags: rust basics

1. What is Rust?
> Rust is a systems programming language focused on safety, speed, and concurrency.

2. What is Cargo?
> Cargo is Rust's package manager and build system.

3. Rust's ownership system prevents {data races} at compile time.

4. What are the three rules of ownership in Rust?
> 1. Each value has a variable called its owner
> 2. There can only be one owner at a time
> 3. When the owner goes out of scope, the value is dropped
---

---
Deck: Mathematics
Tags: algebra formulas

1. What is the quadratic formula?
> $x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$

2. The {Pythagorean theorem} states that $a^2 + b^2 = c^2$.

3. What is the formula for the area of a circle?
> $A = \pi r^2$ where $r$ is the radius
---

## Usage

Process this file with:

```bash
# Process single file
ankiview collect examples/sample-notes.md

# Process with specific collection
ankiview -c /path/to/collection.anki2 collect examples/sample-notes.md

# Process entire directory recursively
ankiview collect ./my-notes --recursive
```

## Format Guide

### Sections
- Sections are delimited by `---`
- Each section can have `Deck:` and `Tags:` metadata
- All cards in a section share the same deck and tags

### Card Types

**Basic Cards** (front/back):
```markdown
1. Question here?
> Answer here
```

**Cloze Deletions** (fill-in-the-blank):
```markdown
1. This is a {cloze deletion} example.
```

### Math Support

Use `$` for inline math: `$E = mc^2$`

Use `$$` for block math:
```markdown
$$
\int_a^b f(x) dx
$$
```

### IDs

After first run, Anki IDs are injected:
```markdown
<!--ID:1234567890-->
1. Question?
> Answer
```

Don't modify IDs - they link to Anki notes for updates.
