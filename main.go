package main

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

var version = "dev"

func main() {
	autoYes := false
	dirPath := ""

	for _, arg := range os.Args[1:] {
		switch arg {
		case "--version":
			fmt.Println("AppImageXdg version", version)
			return
		case "-h", "--help":
			showHelp()
			return
		case "-y":
			autoYes = true
		default:
			if !strings.HasPrefix(arg, "-") && dirPath == "" {
				dirPath = arg
			}
		}
	}

	if dirPath == "" {
		var err error
		dirPath, err = os.Getwd()
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error getting current directory: %v\n", err)
			os.Exit(1)
		}
	}

	info, err := os.Stat(dirPath)
	if err == nil && !info.IsDir() && strings.HasSuffix(dirPath, ".AppImage") {
		cleanupStaleEntries(autoYes)
		installSingleAppImage(dirPath, autoYes)
		return
	}

	cleanupStaleEntries(autoYes)
	installUnintegratedAppImages(dirPath, autoYes)
}

func showHelp() {
	fmt.Println(`AppImageXdg - Manage AppImage desktop integration

Usage:
  AppImageXdg [path] [-y]

  path       Directory containing .AppImage files, or a single .AppImage file
             (defaults to current directory)
  -y         Answer yes to all prompts
  --version  Show version
  -h, --help Show this help

AppImageXdg performs two operations:
  1. Removes stale desktop entries whose executables no longer exist
  2. Creates desktop entries for AppImage files not yet integrated

When a single .AppImage file is provided, AppImageXdg can optionally move
it to ~/Applications before integrating.`)
}

func cleanupStaleEntries(autoYes bool) {
	entries, err := ListAllDesktopEntries()
	if err != nil {
		return
	}

	for _, entry := range entries {
		execPath := ExecLineToPath(entry.Exec)
		if execPath == "" {
			continue
		}
		if !IsExecutable(execPath) {
			fmt.Printf("Stale entry: '%s' -> %s\n", entry.Name, execPath)
			if autoYes || promptYesNo("Remove this entry?") {
				if err := RemoveDesktopEntry(entry); err != nil {
					fmt.Fprintf(os.Stderr, "  Error: %v\n", err)
				} else {
					fmt.Println("  Removed.")
				}
			}
		}
	}
}

func installUnintegratedAppImages(dirPath string, autoYes bool) {
	appImages, _ := filepath.Glob(filepath.Join(dirPath, "*.AppImage"))
	if len(appImages) == 0 {
		return
	}

	for _, app := range appImages {
		if !IsAppImageReferenced(app) {
			fmt.Printf("Unintegrated: %s\n", filepath.Base(app))
			if autoYes || promptYesNo("Install it?") {
				installAppImage(app)
			}
		}
	}
}

func installSingleAppImage(appImagePath string, autoYes bool) {
	if IsAppImageReferenced(appImagePath) {
		fmt.Printf("Already integrated: %s\n", filepath.Base(appImagePath))
		return
	}

	home, _ := os.UserHomeDir()
	homeApplications := filepath.Join(home, "Applications")

	destPath := appImagePath
	if filepath.Dir(appImagePath) != homeApplications {
		fmt.Printf("Not in ~/Applications: %s\n", filepath.Base(appImagePath))
		if autoYes || promptYesNo("Move it to ~/Applications?") {
			os.MkdirAll(homeApplications, 0755)
			newPath := filepath.Join(homeApplications, filepath.Base(appImagePath))
			if err := os.Rename(appImagePath, newPath); err != nil {
				fmt.Fprintf(os.Stderr, "  Error moving file: %v\n", err)
			} else {
				fmt.Printf("  Moved to %s\n", newPath)
				destPath = newPath
			}
		}
	}

	fmt.Printf("Unintegrated: %s\n", filepath.Base(destPath))
	if autoYes || promptYesNo("Install it?") {
		installAppImage(destPath)
	}
}

func installAppImage(appImagePath string) {
	ensureDirs()
	if err := processAppImage(appImagePath); err != nil {
		fmt.Fprintf(os.Stderr, "  Failed: %v\n", err)
	}
}

func processAppImage(appImagePath string) error {
	_ = os.Chmod(appImagePath, 0755)

	fmt.Printf("  Processing %s...\n", filepath.Base(appImagePath))

	mountPoint, pid, err := MountAppImage(appImagePath)
	if err != nil {
		return fmt.Errorf("mount failed: %w", err)
	}
	defer UnmountAppImage(pid, mountPoint)

	desktopContent, iconInMount, err := ExtractDesktopFile(mountPoint)
	if err != nil {
		return fmt.Errorf("extract desktop file: %w", err)
	}

	iconPath := ""
	if iconInMount != "" {
		ip, err := CopyIcon(iconInMount, iconsDir())
		if err == nil {
			iconPath = ip
		}
	}

	desktopContent = ModifyDesktopContent(desktopContent, appImagePath, iconPath)

	if err := WriteDesktopEntry(desktopContent); err != nil {
		return fmt.Errorf("write desktop entry: %w", err)
	}

	name := desktopField(desktopContent, "Name")
	fmt.Printf("  Installed: %s\n", name)
	return nil
}

func promptYesNo(prompt string) bool {
	fmt.Printf("%s (y/n): ", prompt)
	reader := bufio.NewReader(os.Stdin)
	response, _ := reader.ReadString('\n')
	response = strings.TrimSpace(strings.ToLower(response))
	return response == "y" || response == "yes"
}
