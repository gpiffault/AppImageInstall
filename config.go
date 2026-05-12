package main

import (
	"bufio"
	"os"
	"path/filepath"
	"strings"
)

type Config struct {
	IconsDir     string
	UpdateDir    string
	AppImageDirs []string
	ConfigDir    string
}

func expandPath(path string) string {
	home, _ := os.UserHomeDir()
	path = strings.ReplaceAll(path, "~", home)
	path = strings.ReplaceAll(path, "$HOME", home)
	path = os.ExpandEnv(path)
	return path
}

func LoadConfig() *Config {
	home, _ := os.UserHomeDir()
	configDir := filepath.Join(home, ".config", "AppImageXdg")

	cfg := &Config{
		IconsDir:     filepath.Join(home, ".local", "share", "icons", "AppImageXdg"),
		UpdateDir:    filepath.Join(home, ".local", "share", "applications"),
		AppImageDirs: []string{filepath.Join(home, "AppImages"), filepath.Join(home, "Applications")},
		ConfigDir:    configDir,
	}

	configFile := filepath.Join(configDir, "config.ini")
	f, err := os.Open(configFile)
	if err != nil {
		return cfg
	}
	defer f.Close()

	var appimagesDirLegacy string
	var appimagesDirs []string
	hasAppImageDirs := false

	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		eqIdx := strings.Index(line, "=")
		if eqIdx < 0 {
			continue
		}
		key := strings.TrimSpace(line[:eqIdx])
		value := strings.TrimSpace(line[eqIdx+1:])

		switch key {
		case "icons_dir":
			cfg.IconsDir = expandPath(value)
		case "update_dir":
			cfg.UpdateDir = expandPath(value)
		case "appimages_dir":
			appimagesDirLegacy = expandPath(value)
		case "appimages_dirs":
			hasAppImageDirs = true
			if strings.HasPrefix(value, "(") && strings.HasSuffix(value, ")") {
				inner := value[1 : len(value)-1]
				for _, q := range parseQuotedStrings(inner) {
					appimagesDirs = append(appimagesDirs, expandPath(q))
				}
			}
		}
	}

	if hasAppImageDirs && len(appimagesDirs) > 0 {
		cfg.AppImageDirs = appimagesDirs
	} else if appimagesDirLegacy != "" {
		cfg.AppImageDirs = []string{appimagesDirLegacy}
	}

	_ = os.MkdirAll(cfg.UpdateDir, 0755)
	_ = os.MkdirAll(cfg.IconsDir, 0755)

	return cfg
}

func parseQuotedStrings(s string) []string {
	var result []string
	var current strings.Builder
	inQuote := false
	escaped := false

	for _, ch := range s {
		switch {
		case escaped:
			current.WriteRune(ch)
			escaped = false
		case ch == '\\':
			escaped = true
		case ch == '"':
			if inQuote {
				if current.Len() > 0 {
					result = append(result, current.String())
					current.Reset()
				}
			}
			inQuote = !inQuote
		default:
			if inQuote {
				current.WriteRune(ch)
			}
		}
	}
	return result
}

func (c *Config) EnsureDirs() error {
	if err := os.MkdirAll(c.UpdateDir, 0755); err != nil {
		return err
	}
	if err := os.MkdirAll(c.IconsDir, 0755); err != nil {
		return err
	}
	return nil
}
