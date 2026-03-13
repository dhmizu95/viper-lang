def main():
    text = "alpha,beta,gamma,delta;" * 400

    count_a = 0
    count_comma = 0
    count_semicolon = 0
    for ch in text:
        if ch == "a":
            count_a += 1
        elif ch == ",":
            count_comma += 1
        elif ch == ";":
            count_semicolon += 1

    print(len(text) + count_a + count_comma + count_semicolon)


if __name__ == "__main__":
    main()
