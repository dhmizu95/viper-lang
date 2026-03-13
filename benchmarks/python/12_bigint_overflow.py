def main():
    value = 1 << 100
    checksum = 0
    i = 0
    while i < 200000:
        value = value + 123456789
        value = value - 98765432
        checksum += value % 97
        i += 1
    print(checksum)


if __name__ == "__main__":
    main()
