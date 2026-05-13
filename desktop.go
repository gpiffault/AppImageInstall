package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

type DesktopEntry struct {
	Path       string
	Name       string
	Exec       string
	Icon       string
	Version    string
	Categories string
}

func WriteDesktopEntry(content string) error {
	name := desktopField(content, "Name")
	if name == "" {
		return fmt.Errorf("no Name= found in desktop content")
	}

	safeName := strings.ReplaceAll(name, " ", "_")
	desktopPath := filepath.Join(applicationsDir(), safeName+".desktop")
	tempPath := desktopPath + ".tmp"

	if err := os.WriteFile(tempPath, []byte(content), 0644); err != nil {
		return fmt.Errorf("write temp desktop file: %w", err)
	}

	if err := os.Rename(tempPath, desktopPath); err != nil {
		os.Remove(tempPath)
		return fmt.Errorf("rename temp to final: %w", err)
	}

	exec.Command("update-desktop-database", applicationsDir()).Run()
	return nil
}

func desktopField(content, key string) string {
	prefix := key + "="
	scanner := bufio.NewScanner(strings.NewReader(content))
	for scanner.Scan() {
		line := scanner.Text()
		if strings.HasPrefix(line, prefix) {
			return strings.TrimPrefix(line, prefix)
		}
	}
	return ""
}

func ModifyDesktopContent(desktopContent, appImagePath, iconPath string) string {
	var out strings.Builder
	scanner := bufio.NewScanner(strings.NewReader(desktopContent))

	for scanner.Scan() {
		line := scanner.Text()

		if strings.HasPrefix(line, "Exec=") {
			val := strings.TrimPrefix(line, "Exec=")
			if strings.HasPrefix(val, "AppRun") {
				val = strings.Replace(val, "AppRun", fmt.Sprintf(`"%s"`, appImagePath), 1)
			} else {
				val = fmt.Sprintf(`"%s" %%U`, appImagePath)
			}
			line = "Exec=" + val
		}

		if strings.HasPrefix(line, "Icon=") && iconPath != "" {
			line = "Icon=" + iconPath
		}

		out.WriteString(line)
		out.WriteByte('\n')
	}

	return out.String()
}

func ListAllDesktopEntries() ([]DesktopEntry, error) {
	var entries []DesktopEntry

	matches, err := filepath.Glob(filepath.Join(applicationsDir(), "*.desktop"))
	if err != nil {
		return nil, err
	}

	for _, path := range matches {
		entry := parseDesktopEntry(path)
		if entry != nil {
			entries = append(entries, *entry)
		}
	}
	return entries, nil
}

func parseDesktopEntry(path string) *DesktopEntry {
	f, err := os.Open(path)
	if err != nil {
		return nil
	}
	defer f.Close()

	entry := &DesktopEntry{Path: path}
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := scanner.Text()
		switch {
		case strings.HasPrefix(line, "Name="):
			entry.Name = strings.TrimPrefix(line, "Name=")
		case strings.HasPrefix(line, "Exec="):
			entry.Exec = strings.TrimPrefix(line, "Exec=")
		case strings.HasPrefix(line, "Icon="):
			entry.Icon = strings.TrimPrefix(line, "Icon=")
		case strings.HasPrefix(line, "Version="):
			entry.Version = strings.TrimPrefix(line, "Version=")
		case strings.HasPrefix(line, "Categories="):
			entry.Categories = strings.TrimPrefix(line, "Categories=")
		}
	}
	return entry
}

func IsAppImageReferenced(appImagePath string) bool {
	entries, _ := ListAllDesktopEntries()
	baseName := strings.TrimSuffix(filepath.Base(appImagePath), filepath.Ext(appImagePath))

	for _, e := range entries {
		if strings.Contains(strings.ToLower(e.Exec), strings.ToLower(baseName)) {
			return true
		}
	}
	return false
}

func RemoveDesktopEntry(entry DesktopEntry) error {
	if entry.Icon != "" &&
		(strings.Contains(entry.Icon, "AppImageXdg") ||
			strings.Contains(entry.Icon, "/icons/")) {
		os.Remove(entry.Icon)
	}

	if err := os.Remove(entry.Path); err != nil {
		return err
	}

	exec.Command("update-desktop-database", applicationsDir()).Run()
	return nil
}

func ExecLineToPath(execLine string) string {
	var result strings.Builder
	inQuote := false
	for _, ch := range execLine {
		if ch == '"' {
			inQuote = !inQuote
			continue
		}
		if ch == ' ' && !inQuote {
			if result.Len() > 0 {
				break
			}
			continue
		}
		result.WriteRune(ch)
	}
	return result.String()
}
