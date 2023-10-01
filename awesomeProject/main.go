package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"os"
)

func main() {
	q := [2][2]int{{2, 3}}
	fmt.Print(q)
	f, err := os.Open("thing.txt")
	if err != nil {
		log.Fatalf("failed to open file: %v", err)
	}
	defer func() {
		if fErr := f.Close(); fErr != nil {
			log.Fatalf("failed to close file: %v!", fErr)
		}
	}()

	out, oErr := os.Create("outthing.txt")
	if oErr != nil {
		log.Fatalf("failed to create out file")
	}
	defer func() {
		if clErr := out.Close(); clErr != nil {
			log.Fatalf("failed to close created file")
		}
	}()

	buffer := make([]uint8, 1024)
	for {
		size, cErr := f.Read(buffer)
		if cErr != nil && cErr != io.EOF {
			log.Fatalf("unexpected error when reading to buffer: %v", cErr)
		}
		if size == 0 {
			break
		}
		if _, aErr := out.Write(buffer); aErr != nil {
			log.Fatalf("failed to write to out file: %v", aErr)
		}
	}

	secondIn, inErr := os.Open("fart.txt")
	if inErr != nil {
		panic(inErr)
	}

	secondOut, secondErr := os.Create("newOut.txt")
	if secondErr != nil {
		panic(secondErr)
	}

	r := bufio.NewReader(secondIn)
	w := bufio.NewWriter(secondOut)

	for {
		size, sErr := r.Read(buffer)
		if sErr != nil && sErr != io.EOF {
			panic(sErr)
		}
		if size == 0 {
			break
		}
		_, wErr := w.Write(buffer)
		if wErr != nil {
			panic(wErr)
		}
	}
}
