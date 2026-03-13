def main():
    n = 50
    a = [0] * (n * n)
    b = [0] * (n * n)
    c = [0] * (n * n)

    i = 0
    while i < n:
        j = 0
        while j < n:
            idx = i * n + j
            a[idx] = (i + j) % 10
            b[idx] = (i * j) % 10
            j += 1
        i += 1

    i = 0
    while i < n:
        j = 0
        while j < n:
            sum_val = 0
            k = 0
            while k < n:
                sum_val += a[i * n + k] * b[k * n + j]
                k += 1
            c[i * n + j] = sum_val
            j += 1
        i += 1

    print(sum(c))


if __name__ == "__main__":
    main()
