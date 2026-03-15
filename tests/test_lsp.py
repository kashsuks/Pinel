"""
LSP Test File for Pinel Editor
================================
Open this file in Pinel to verify that LSP (pyright) features work:

  • Hover docs  – hover your mouse over any symbol to see its type / docstring
  • Autocomplete – start typing a symbol name; the completion list should appear
  • Go-to-def   – Ctrl+click (or the command-palette action) on a symbol

Quick autocomplete tests to try  (type the partial word and wait ~300 ms):
  - `os.path.j`       → should suggest  join, isabs, …
  - `json.dum`        → should suggest  dumps, dump
  - `MyAnimal.`       → should suggest  name, sound, speak, …
  - `dataclass`       → keyword / type completion
  - `Optio`           → should suggest  Optional  (from typing)
  - `re.com`          → should suggest  compile
  - `Greeting(`       → hover shows the signature / docstring
"""

from __future__ import annotations

import datetime
import json
import math
import os
import pathlib
import re
from abc import ABC, abstractmethod
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple, Union

# ---------------------------------------------------------------------------
# 1. Dataclasses  – hover over field names and types to see pyright info
# ---------------------------------------------------------------------------


@dataclass
class Point:
    """A 2-D point with optional label."""

    x: float
    y: float
    label: str = ""

    def distance_to(self, other: "Point") -> float:
        """Return the Euclidean distance between this point and *other*."""
        return math.sqrt((self.x - other.x) ** 2 + (self.y - other.y) ** 2)

    def midpoint(self, other: "Point") -> "Point":
        return Point((self.x + other.x) / 2, (self.y + other.y) / 2)

    def __repr__(self) -> str:
        label_part = f", label={self.label!r}" if self.label else ""
        return f"Point(x={self.x}, y={self.y}{label_part})"


@dataclass
class BoundingBox:
    """Axis-aligned bounding box defined by two corners."""

    top_left: Point
    bottom_right: Point
    tags: List[str] = field(default_factory=list)

    @property
    def width(self) -> float:
        return abs(self.bottom_right.x - self.top_left.x)

    @property
    def height(self) -> float:
        return abs(self.bottom_right.y - self.top_left.y)

    @property
    def area(self) -> float:
        return self.width * self.height

    def contains(self, p: Point) -> bool:
        """Return True if *p* falls inside or on the boundary of the box."""
        return (
            self.top_left.x <= p.x <= self.bottom_right.x
            and self.top_left.y <= p.y <= self.bottom_right.y
        )


# ---------------------------------------------------------------------------
# 2. Abstract base class + concrete subclasses
#    Try typing  `animal.`  to get method completions
# ---------------------------------------------------------------------------


class Animal(ABC):
    """Abstract base for all animals."""

    def __init__(self, name: str, age: int) -> None:
        self.name = name
        self.age = age
        self._health: float = 100.0

    @abstractmethod
    def speak(self) -> str:
        """Return the sound this animal makes."""
        ...

    @property
    def health(self) -> float:
        return self._health

    @health.setter
    def health(self, value: float) -> None:
        self._health = max(0.0, min(100.0, value))

    def birthday(self) -> None:
        """Increment the animal's age by one year."""
        self.age += 1

    def describe(self) -> str:
        return f"{self.name} is {self.age} years old and says: {self.speak()}"


class Dog(Animal):
    """A domestic dog with a breed attribute."""

    def __init__(self, name: str, age: int, breed: str) -> None:
        super().__init__(name, age)
        self.breed = breed
        self.tricks: List[str] = []

    def speak(self) -> str:
        return "Woof!"

    def learn_trick(self, trick: str) -> None:
        """Teach the dog a new trick."""
        if trick not in self.tricks:
            self.tricks.append(trick)

    def show_tricks(self) -> str:
        if not self.tricks:
            return f"{self.name} knows no tricks yet."
        return f"{self.name} can: {', '.join(self.tricks)}"


class Cat(Animal):
    """A domestic cat, optionally indoor-only."""

    def __init__(self, name: str, age: int, indoor: bool = True) -> None:
        super().__init__(name, age)
        self.indoor = indoor

    def speak(self) -> str:
        return "Meow!"

    def purr(self) -> str:
        return "Purrr..."


# ---------------------------------------------------------------------------
# 3. Generic / typed collections
#    Hover over return types to see the inferred signatures
# ---------------------------------------------------------------------------


def group_by_first_letter(words: List[str]) -> Dict[str, List[str]]:
    """Group *words* into a dict keyed by their first (lower-case) letter."""
    result: Dict[str, List[str]] = defaultdict(list)
    for word in words:
        if word:
            result[word[0].lower()].append(word)
    return dict(result)


def most_common_words(text: str, n: int = 10) -> List[Tuple[str, int]]:
    """Return the *n* most common whitespace-delimited words in *text*."""
    tokens = re.findall(r"[a-zA-Z']+", text.lower())
    counter = Counter(tokens)
    return counter.most_common(n)


def parse_csv_line(line: str, sep: str = ",") -> List[str]:
    """Split *line* by *sep* and strip surrounding whitespace from each field."""
    return [field.strip() for field in line.split(sep)]


# ---------------------------------------------------------------------------
# 4. File-system helpers  (uses os, pathlib – good for  os.path.  completions)
#    Try:  `os.path.` , `pathlib.Path(` , `p.`
# ---------------------------------------------------------------------------


def list_python_files(directory: str) -> List[pathlib.Path]:
    """Return all  .py  files found (non-recursively) inside *directory*."""
    base = pathlib.Path(directory)
    if not base.is_dir():
        return []
    return sorted(base.glob("*.py"))


def read_json_file(path: str) -> Optional[Any]:
    """Read and return the JSON value stored in *path*, or None on error."""
    full_path = os.path.abspath(os.path.expanduser(path))
    if not os.path.isfile(full_path):
        return None
    try:
        with open(full_path, encoding="utf-8") as fh:
            return json.load(fh)
    except (json.JSONDecodeError, OSError):
        return None


def ensure_directory(path: Union[str, pathlib.Path]) -> pathlib.Path:
    """Create *path* (and parents) if it does not exist; return the Path."""
    p = pathlib.Path(path)
    p.mkdir(parents=True, exist_ok=True)
    return p


# ---------------------------------------------------------------------------
# 5. Date / time utilities
#    Hover over `datetime.date.` for member completions
# ---------------------------------------------------------------------------


def days_until(target: datetime.date) -> int:
    """Return the number of days from today until *target* (negative if past)."""
    return (target - datetime.date.today()).days


def format_duration(seconds: float) -> str:
    """Format a duration given in fractional *seconds* as  HH:MM:SS.mmm."""
    total_ms = int(round(seconds * 1000))
    ms = total_ms % 1000
    total_s = total_ms // 1000
    s = total_s % 60
    total_m = total_s // 60
    m = total_m % 60
    h = total_m // 60
    return f"{h:02d}:{m:02d}:{s:02d}.{ms:03d}"


# ---------------------------------------------------------------------------
# 6. Regular expressions
#    Try:  `re.`  for compile, match, search, findall, sub, …
# ---------------------------------------------------------------------------

EMAIL_PATTERN: re.Pattern[str] = re.compile(
    r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$"
)

URL_PATTERN: re.Pattern[str] = re.compile(r"https?://[^\s/$.?#].[^\s]*")


def is_valid_email(address: str) -> bool:
    """Return True if *address* looks like a valid e-mail address."""
    return bool(EMAIL_PATTERN.match(address))


def extract_urls(text: str) -> List[str]:
    """Return every URL found in *text*."""
    return URL_PATTERN.findall(text)


# ---------------------------------------------------------------------------
# 7. Exception hierarchy  (custom errors)
# ---------------------------------------------------------------------------


class PinelError(Exception):
    """Base exception for Pinel-related errors."""


class ConfigError(PinelError):
    """Raised when the configuration is invalid or missing."""

    def __init__(self, key: str, reason: str = "") -> None:
        self.key = key
        self.reason = reason
        msg = f"Config error for key {key!r}"
        if reason:
            msg += f": {reason}"
        super().__init__(msg)


class FileNotSupportedError(PinelError):
    """Raised when a file type is not supported by the editor."""

    def __init__(self, extension: str) -> None:
        self.extension = extension
        super().__init__(f"File extension {extension!r} is not supported")


# ---------------------------------------------------------------------------
# 8. Context manager
# ---------------------------------------------------------------------------


class Timer:
    """Simple wall-clock timer used as a context manager."""

    def __init__(self, label: str = "") -> None:
        self.label = label
        self._start: Optional[float] = None
        self.elapsed: float = 0.0

    def __enter__(self) -> "Timer":
        import time

        self._start = time.perf_counter()
        return self

    def __exit__(self, *_: Any) -> None:
        import time

        if self._start is not None:
            self.elapsed = time.perf_counter() - self._start
        if self.label:
            print(f"{self.label}: {format_duration(self.elapsed)}")


# ---------------------------------------------------------------------------
# 9. Generator / iterator
# ---------------------------------------------------------------------------


def fibonacci(limit: int):
    """Yield Fibonacci numbers up to (but not exceeding) *limit*."""
    a, b = 0, 1
    while a <= limit:
        yield a
        a, b = b, a + b


def sliding_window(seq: List[Any], size: int):
    """Yield overlapping windows of *size* from *seq*."""
    for i in range(len(seq) - size + 1):
        yield seq[i : i + size]


# ---------------------------------------------------------------------------
# 10. Module-level "smoke test"  – run with  python tests/test_lsp.py
# ---------------------------------------------------------------------------


def _smoke_test() -> None:
    # Points and bounding box
    origin = Point(0.0, 0.0, "origin")
    corner = Point(3.0, 4.0, "corner")
    print("Distance:", origin.distance_to(corner))  # 5.0

    box = BoundingBox(Point(0, 0), Point(10, 10), tags=["test"])
    print("Box area:", box.area)  # 100.0
    print("Contains origin:", box.contains(origin))  # True

    # Animals
    dog = Dog("Rex", 3, "Labrador")
    dog.learn_trick("sit")
    dog.learn_trick("shake")
    print(dog.show_tricks())
    print(dog.describe())

    cat = Cat("Whiskers", 5)
    print(cat.describe())

    # Text utilities
    words = ["apple", "banana", "avocado", "blueberry", "cherry"]
    grouped = group_by_first_letter(words)
    print("Grouped:", grouped)

    # Date helpers
    new_year = datetime.date(datetime.date.today().year + 1, 1, 1)
    print("Days until New Year:", days_until(new_year))

    # Duration formatting
    print("Duration:", format_duration(3723.5))  # 01:02:03.500

    # Regex
    print("Valid email:", is_valid_email("user@example.com"))
    print("Invalid email:", is_valid_email("not-an-email"))

    # Fibonacci
    fibs = list(fibonacci(100))
    print("Fibonacci ≤ 100:", fibs)

    # Sliding window
    windows = list(sliding_window([1, 2, 3, 4, 5], 3))
    print("Windows:", windows)

    # Timer context manager
    with Timer("smoke test") as t:
        _ = sum(fibonacci(10_000))
    print(f"Timer elapsed: {t.elapsed:.6f}s")


if __name__ == "__main__":
    _smoke_test()
