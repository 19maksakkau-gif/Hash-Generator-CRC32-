// crc32.go
package main

import (
	"bufio"
	"encoding/binary"
	"flag"
	"fmt"
	"hash/crc32"
	"io"
	"os"
	"strconv"
	"strings"
)

func computeCRC32(data []byte, useCRC32C bool) uint32 {
	if useCRC32C {
		return crc32.Checksum(data, crc32.MakeTable(crc32.Castagnoli))
	}
	return crc32.ChecksumIEEE(data)
}

func computeFileCRC32(filename string, useCRC32C bool) (uint32, error) {
	file, err := os.Open(filename)
	if err != nil {
		return 0, err
	}
	defer file.Close()
	var table *crc32.Table
	if useCRC32C {
		table = crc32.MakeTable(crc32.Castagnoli)
	} else {
		table = crc32.IEEETable
	}
	hash := crc32.New(table)
	if _, err := io.Copy(hash, file); err != nil {
		return 0, err
	}
	return hash.Sum32(), nil
}

func formatHash(val uint32, format string) string {
	switch format {
	case "hex":
		return fmt.Sprintf("0x%08X", val)
	case "dec":
		return fmt.Sprintf("%d", val)
	case "bin":
		return fmt.Sprintf("%032b", val)
	default:
		return fmt.Sprintf("0x%08X", val)
	}
}

func main() {
	var (
		useCRC32C bool
		check     string
		dec       bool
		bin       bool
		progress  bool
	)
	flag.BoolVar(&useCRC32C, "crc32c", false, "Использовать CRC32C")
	flag.StringVar(&check, "check", "", "Сравнить с указанным хэшем (HEX)")
	flag.BoolVar(&dec, "dec", false, "Вывод в десятичном формате")
	flag.BoolVar(&bin, "bin", false, "Вывод в бинарном формате")
	flag.BoolVar(&progress, "progress", false, "Показывать прогресс")
	flag.Parse()
	args := flag.Args()

	format := "hex"
	if dec {
		format = "dec"
	} else if bin {
		format = "bin"
	}

	// Если нет аргументов и есть данные в stdin
	if len(args) == 0 {
		stat, _ := os.Stdin.Stat()
		if (stat.Mode() & os.ModeCharDevice) == 0 {
			data, err := io.ReadAll(os.Stdin)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Ошибка чтения stdin: %v\n", err)
				os.Exit(1)
			}
			hashVal := computeCRC32(data, useCRC32C)
			fmt.Println(formatHash(hashVal, format))
			return
		}
		fmt.Println("Не указаны данные. Используйте: crc32 <строка или файл>")
		os.Exit(1)
	}

	for _, item := range args {
		info, err := os.Stat(item)
		if err == nil && !info.IsDir() {
			// Это файл
			hashVal, err := computeFileCRC32(item, useCRC32C)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Ошибка чтения %s: %v\n", item, err)
				continue
			}
			output := fmt.Sprintf("%s: %s", item, formatHash(hashVal, format))
			if check != "" {
				expected, _ := strconv.ParseUint(strings.TrimPrefix(check, "0x"), 16, 32)
				status := "✅ OK"
				if uint32(expected) != hashVal {
					status = "❌ FAIL"
				}
				output += fmt.Sprintf(" (check: %s)", status)
			}
			fmt.Println(output)
		} else {
			// Строка
			data := []byte(item)
			hashVal := computeCRC32(data, useCRC32C)
			output := formatHash(hashVal, format)
			if check != "" {
				expected, _ := strconv.ParseUint(strings.TrimPrefix(check, "0x"), 16, 32)
				status := "✅ OK"
				if uint32(expected) != hashVal {
					status = "❌ FAIL"
				}
				output += fmt.Sprintf(" (check: %s)", status)
			}
			fmt.Println(output)
		}
	}
}
