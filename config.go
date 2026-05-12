package main

import (
	"os"
	"path/filepath"
)

func xdgDataHome() string {
	if dir := os.Getenv("XDG_DATA_HOME"); dir != "" {
		return dir
	}
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".local", "share")
}

func iconsDir() string {
	return filepath.Join(xdgDataHome(), "icons", "AppImageXdg")
}

func applicationsDir() string {
	return filepath.Join(xdgDataHome(), "applications")
}

func ensureDirs() error {
	if err := os.MkdirAll(applicationsDir(), 0755); err != nil {
		return err
	}
	if err := os.MkdirAll(iconsDir(), 0755); err != nil {
		return err
	}
	return nil
}
