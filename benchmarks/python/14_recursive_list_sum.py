def sum_range(n):
    if n <= 0:
        return 0
    return n + sum_range(n - 1)


if __name__ == "__main__":
    # Keep recursion depth comfortably below typical stack limits
    n = 200
    print(sum_range(n))
