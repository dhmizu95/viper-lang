def main():
    n = 30
    checksum = 0
    row = 0
    while row < n:
        col = 0
        while col < n:
            sum_val = 0
            k = 0
            while k < n:
                a_val = (row * n + k) % 10
                b_val = (k * n + col + 1) % 10
                sum_val += a_val * b_val
                k += 1
            checksum += sum_val
            col += 1
        row += 1
    print(checksum)


if __name__ == "__main__":
    main()
