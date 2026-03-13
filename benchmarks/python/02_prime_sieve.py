def is_prime(n):
    if n < 2:
        return 0
    if n == 2:
        return 1
    if n % 2 == 0:
        return 0
    i = 3
    while i * i <= n:
        if n % i == 0:
            return 0
        i += 2
    return 1


def count_primes(n):
    count = 0
    i = 2
    while i <= n:
        if is_prime(i) == 1:
            count += 1
        i += 1
    return count


if __name__ == "__main__":
    print(count_primes(5000))
