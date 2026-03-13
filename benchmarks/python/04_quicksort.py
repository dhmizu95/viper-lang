def main():
    n = 100
    arr = [0] * n
    i = 0
    seed = 12345
    while i < n:
        seed = (seed * 1103515245 + 12345) % 2147483648
        arr[i] = seed % 1000
        i += 1

    stack = [0] * 200
    top = 0
    stack[top] = 0
    top += 1
    stack[top] = n - 1
    top += 1

    while top > 1:
        top -= 1
        high = stack[top]
        top -= 1
        low = stack[top]
        if low < high:
            pivot = arr[high]
            partition_idx = low
            j = low
            while j < high:
                if arr[j] < pivot:
                    arr[partition_idx], arr[j] = arr[j], arr[partition_idx]
                    partition_idx += 1
                j += 1
            arr[partition_idx], arr[high] = arr[high], arr[partition_idx]
            if partition_idx - 1 > low:
                stack[top] = low
                top += 1
                stack[top] = partition_idx - 1
                top += 1
            if partition_idx + 1 < high:
                stack[top] = partition_idx + 1
                top += 1
                stack[top] = high
                top += 1

    print(sum(arr))


if __name__ == "__main__":
    main()
