package main

import "fmt"

func main() {
    sum := int64(0)
    i := int64(0)
    for i < 1000000000 {
        sum = sum + i
        i++
    }
    fmt.Println(sum)
}
