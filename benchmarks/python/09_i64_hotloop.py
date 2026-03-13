def main():
    n = 2000000
    acc = 1
    i = 1
    while i <= n:
        acc += i
        acc -= i % 7
        acc += (i * 3) % 11
        if i % 5 == 0:
            acc = acc // 2 + 17
        i += 1
    print(acc)


if __name__ == "__main__":
    main()
