def sum_list(lst, idx):
    if idx >= len(lst):
        return 0
    return lst[idx] + sum_list(lst, idx + 1)


if __name__ == "__main__":
    lst = list(range(1, 1001))
    print(sum_list(lst, 0))