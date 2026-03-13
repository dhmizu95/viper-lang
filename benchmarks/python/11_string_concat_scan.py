def main():
    parts = []
    i = 0
    while i < 250:
        parts.append("item=" + str(i) + ";")
        i += 1

    text = "".join(parts)
    digits = 0
    equals = 0
    semicolons = 0
    for ch in text:
        if "0" <= ch <= "9":
            digits += 1
        elif ch == "=":
            equals += 1
        elif ch == ";":
            semicolons += 1

    print(len(text) + digits + equals + semicolons)


if __name__ == "__main__":
    main()
