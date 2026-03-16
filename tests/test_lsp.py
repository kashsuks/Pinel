from __future__ import annotations

from dataclasses import dataclass
from typing import Callable, Iterable, TypeVar

T = TypeVar("T")
U = TypeVar("U")


class Math:
    @staticmethod
    def fibonacci(n: int) -> int:
        if n <= 1:
            return n
        return Math.fibonacci(n - 1) + Math.fibonacci(n - 2)

    @staticmethod
    def factorial(n: int) -> int:
        result = 1
        for i in range(1, n + 1):
            result *= i
        return result

    @staticmethod
    def is_prime(n: int) -> bool:
        if n <= 1:
            return False
        if n <= 3:
            return True
        if n % 2 == 0 or n % 3 == 0:
            return False
        i = 5
        while i * i <= n:
            if n % i == 0 or n % (i + 2) == 0:
                return False
            i += 6
        return True

    @staticmethod
    def gcd(a: int, b: int) -> int:
        x, y = a, b
        while y != 0:
            x, y = y, x % y
        return x

    @staticmethod
    def lcm(a: int, b: int) -> int:
        return a // Math.gcd(a, b) * b

    @staticmethod
    def power(base: int, exp: int) -> int:
        result = 1
        for _ in range(exp):
            result *= base
        return result

    @staticmethod
    def sqrt(n: float) -> float:
        return n**0.5

    @staticmethod
    def abs(n: int) -> int:
        return abs(n)


class StringUtils:
    @staticmethod
    def reverse(value: str) -> str:
        return value[::-1]

    @staticmethod
    def is_palindrome(value: str) -> bool:
        filtered = "".join(ch for ch in value if not ch.isspace())
        return filtered == filtered[::-1]

    @staticmethod
    def capitalize(value: str) -> str:
        return value[:1].upper() + value[1:].lower()

    @staticmethod
    def split(value: str, delimiter: str) -> list[str]:
        return value.split(delimiter)

    @staticmethod
    def trim(value: str) -> str:
        return value.strip()

    @staticmethod
    def contains(value: str, needle: str) -> bool:
        return needle in value

    @staticmethod
    def count(value: str, needle: str) -> int:
        return value.count(needle)


class ArrayUtils:
    @staticmethod
    def map(items: Iterable[T], fn: Callable[[T], U]) -> list[U]:
        return [fn(item) for item in items]

    @staticmethod
    def filter(items: Iterable[T], predicate: Callable[[T], bool]) -> list[T]:
        return [item for item in items if predicate(item)]

    @staticmethod
    def reduce(items: Iterable[T], fn: Callable[[T, T], T]) -> T | None:
        iterator = iter(items)
        try:
            result = next(iterator)
        except StopIteration:
            return None
        for item in iterator:
            result = fn(result, item)
        return result

    @staticmethod
    def find(items: Iterable[T], predicate: Callable[[T], bool]) -> T | None:
        for item in items:
            if predicate(item):
                return item
        return None

    @staticmethod
    def contains(items: Iterable[T], value: T) -> bool:
        return any(item == value for item in items)

    @staticmethod
    def reverse(items: Iterable[T]) -> list[T]:
        return list(reversed(list(items)))

    @staticmethod
    def unique(items: Iterable[T]) -> list[T]:
        seen: set[T] = set()
        result: list[T] = []
        for item in items:
            if item not in seen:
                seen.add(item)
                result.append(item)
        return result

    @staticmethod
    def sum(items: Iterable[int]) -> int:
        return sum(items)


@dataclass
class User:
    name: str
    score: int


def build_user_map(users: list[User]) -> dict[str, int]:
    return {user.name: user.score for user in users}


def main() -> None:
    values = [1, 2, 3, 4, 5, 6, 7]
    squares = ArrayUtils.map(values, lambda v: v * v)
    primes = ArrayUtils.filter(values, Math.is_prime)
    total = ArrayUtils.sum(values)
    greeting = StringUtils.capitalize("salut à tous")
    ratio = Math.sqrt(42.0) / 3.0
    users = [User("alice", 3), User("bob", 5), User("carol", 7)]
    user_map = build_user_map(users)
    _summary = (squares, primes, total, greeting, ratio, user_map)


if __name__ == "__main__":
    main()