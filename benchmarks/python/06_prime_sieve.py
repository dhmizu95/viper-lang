def main():
    n = 10000
    sieve = [1] * (n + 1)
    sieve[0] = 0
    sieve[1] = 0

    p = 2
    while p * p <= n:
        if sieve[p] == 1:
            i = p * p
            while i <= n:
                sieve[i] = 0
                i += p
        p += 1

    count = 0
    i = 0
    while i <= n:
        if sieve[i] == 1:
            count += 1
        i += 1

    print(count)


if __name__ == "__main__":
    main()
