package main

import (
	"bufio"
	"fmt"
	"os"
	"regexp"
	"strings"
)

var (
	reVersion       = regexp.MustCompile(`[-_]v?\d+(\.\d+)*.*$`)
	rePlatform      = regexp.MustCompile(`[-_](linux|x86_64|amd64|i386|arm64|armhf).*$`)
	reDate          = regexp.MustCompile(`[-_]\d{4}[-_]\d{2}[-_]\d{2}.*$`)
	reGitHash       = regexp.MustCompile(`[-_]git[-_][a-f0-9]{7,}.*$`)
	reCleanEdges    = regexp.MustCompile(`^[^a-zA-Z0-9]+|[^a-zA-Z0-9]+$`)
	reSeparators    = regexp.MustCompile(`[-_]+`)
)

func CleanAppImageName(filename string) string {
	name := filename

	for _, ext := range []string{".AppImage", ".appimage", ".APPIMAGE"} {
		name = strings.TrimSuffix(name, ext)
	}

	name = reVersion.ReplaceAllString(name, "")
	name = rePlatform.ReplaceAllString(name, "")
	name = reDate.ReplaceAllString(name, "")
	name = reGitHash.ReplaceAllString(name, "")

	name = reCleanEdges.ReplaceAllString(name, "")
	name = reSeparators.ReplaceAllString(name, " ")

	words := strings.Fields(name)
	for i, w := range words {
		if len(w) > 0 {
			words[i] = strings.ToUpper(w[:1]) + strings.ToLower(w[1:])
		}
	}
	cleaned := strings.Join(words, " ")

	if strings.TrimSpace(cleaned) == "" {
		cleaned = "AppImage"
	}

	return cleaned
}

func PromptForName(suggested string) string {
	fmt.Printf("Use the name [%s]? (y/n): ", suggested)

	reader := bufio.NewReader(os.Stdin)
	response, _ := reader.ReadString('\n')
	response = strings.TrimSpace(strings.ToLower(response))

	if response == "n" || response == "no" {
		fmt.Print("Enter a custom name for the AppImage: ")
		custom, _ := reader.ReadString('\n')
		custom = strings.TrimSpace(custom)
		if custom != "" {
			return custom
		}
		fmt.Fprintf(os.Stderr, "Name cannot be empty. Using default: %s\n", suggested)
	}
	return suggested
}
