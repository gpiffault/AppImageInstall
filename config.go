package main

import (
	"bufio"
	"os"
	"path/filepath"
	"strings"
)

type Config struct {
	IconsDir  string
	UpdateDir string
	ConfigDir string
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
		IconsDir:  filepath.Join(home, ".local", "share", "icons", "AppImageXdg"),
		UpdateDir: filepath.Join(home, ".local", "share", "applications"),
		ConfigDir: configDir,
	}

	configFile := filepath.Join(configDir, "config.ini")
	f, err := os.Open(configFile)
	if err != nil {
		return cfg
	}
	defer f.Close()

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
		}
	}

	_ = os.MkdirAll(cfg.UpdateDir, 0755)
	_ = os.MkdirAll(cfg.IconsDir, 0755)

	return cfg
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
