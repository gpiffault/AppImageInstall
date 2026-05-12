package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

var version = "dev"

func main() {
	args := os.Args[1:]
	cmd := "help"
	if len(args) > 0 {
		cmd = args[0]
		args = args[1:]
	}

	// Direct .AppImage file path → install it
	if strings.HasSuffix(strings.ToLower(cmd), ".appimage") {
		if _, err := os.Stat(cmd); err == nil {
			installSingleAppImage(cmd)
			return
		}
	}

	switch cmd {
	case "status", "info":
		cmdStatus()
	case "find", "search":
		cmdFind()
	case "install", "add":
		cmdInstall(args)
	case "list", "ls":
		cmdList()
	case "remove", "uninstall", "rm":
		cmdRemove(args)
	case "run":
		cmdRun(args)
	case "debug":
		cmdDebug(args)
	case "desktop", "desktops":
		cmdDesktop()
	case "--version", "-v":
		fmt.Printf("AppImageXdg v%s\n", version)
	case "--dry-run":
		fmt.Printf("Dry run mode - would execute: %v\n", os.Args[1:])
		fmt.Println("Note: Dry run functionality not fully implemented yet")
	case "help", "-h", "--help", "":
		showHelp()
	default:
		fmt.Printf("Unknown command: %s\n\n", cmd)
		showHelp()
	}
}

func showHelp() {
	fmt.Println(`AppImageXdg - Simple AppImage Management

Quick Commands:
  AppImageXdg                    Show this help
  AppImageXdg status             Show current configuration and status
  AppImageXdg find               Find AppImages on your system
  AppImageXdg install [file]     Install AppImage(s) - prompts if no file given
  AppImageXdg list               List integrated AppImages
  AppImageXdg remove <n>         Remove an integrated AppImage
  AppImageXdg run <name>         Run an AppImage with live output
  AppImageXdg debug <name>       Run an AppImage with debug/verbose output
  AppImageXdg desktop            Show .desktop files created
  AppImageXdg help               Show detailed help with all options

Examples:
  AppImageXdg find               # Find all AppImages on your system
  AppImageXdg install            # Interactive install from found AppImages
  AppImageXdg install ~/Downloads/app.AppImage
  AppImageXdg remove Firefox     # Remove Firefox integration`)
}

func cmdStatus() {
	fmt.Println("AppImageXdg - Current Status")
	fmt.Println("===========================================")
	fmt.Println()
	fmt.Println("Configuration:")
	fmt.Printf("  Icons stored in: %s\n", iconsDir())
	fmt.Printf("  Desktop entries in: %s\n", applicationsDir())
	fmt.Println()

	entries, _ := ListAppImageDesktopEntries(applicationsDir())
	fmt.Printf("Integrated AppImages: %d\n", len(entries))
	fmt.Println()
	fmt.Println("Quick tips:")
	fmt.Println("  - Run 'AppImageXdg' for short commands")
	fmt.Println("  - Run 'AppImageXdg find' to search for AppImages on your system")
	fmt.Println("  - Run 'AppImageXdg install' to install from common locations")
	fmt.Println("  - Tab completion available: AppImageXdg <TAB>")
}

func cmdFind() {
	cwd, _ := os.Getwd()
	fmt.Printf("Searching for AppImages in %s...\n", cwd)
	fmt.Println("=========================================")
	fmt.Println()

	appImages := findAppImageFiles(cwd)
	if len(appImages) == 0 {
		fmt.Println("No AppImages found in current directory.")
		return
	}

	foundAny := false
	var unintegrated []string
	for _, app := range appImages {
		base := filepath.Base(app)
		if DesktopFileExists(applicationsDir(), base) {
			fmt.Printf("  ✓ %s (already integrated)\n", base)
			foundAny = true
		} else {
			fmt.Printf("  - %s\n", base)
			unintegrated = append(unintegrated, app)
			foundAny = true
		}
	}

	if !foundAny {
		return
	}

	if len(unintegrated) > 0 {
		fmt.Println()
		fmt.Print("Would you like to integrate the unintegrated AppImages? (y/n): ")
		reader := bufio.NewReader(os.Stdin)
		response, _ := reader.ReadString('\n')
		response = strings.TrimSpace(strings.ToLower(response))

		if response == "y" || response == "yes" {
			for _, app := range unintegrated {
				fmt.Println()
				fmt.Printf("Integrating: %s\n", filepath.Base(app))
				installSingleAppImage(app)
			}
		}
	}
}

func findAppImageFiles(dir string) []string {
	var results []string
	// Walk up to 2 levels deep
	filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}
		if info.IsDir() {
			rel, _ := filepath.Rel(dir, path)
			depth := len(strings.Split(rel, string(filepath.Separator)))
			if depth > 2 {
				return filepath.SkipDir
			}
			return nil
		}
		if strings.HasSuffix(strings.ToLower(info.Name()), ".appimage") {
			results = append(results, path)
		}
		return nil
	})
	return results
}

func cmdInstall(args []string) {
	if len(args) == 0 {
		cmdFind()
		return
	}

	for _, appImage := range args {
		if _, err := os.Stat(appImage); err != nil {
			fmt.Printf("File not found: %s\n", appImage)
			continue
		}
		installSingleAppImage(appImage)
	}
}

func installSingleAppImage(appImagePath string) {
	ensureDirs()

	if err := processAppImage(appImagePath); err != nil {
		fmt.Fprintf(os.Stderr, "Processing failed - AppImage remains at: %s\n", appImagePath)
	}
}

func processAppImage(appImagePath string) error {
	_ = os.Chmod(appImagePath, 0755)

	fmt.Printf("Processing %s...\n", filepath.Base(appImagePath))

	mountPoint, pid, err := MountAppImage(appImagePath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to mount AppImage\n")
		return err
	}
	defer UnmountAppImage(pid, mountPoint)

	var execCommand string
	needsNoSandbox := IsElectronApp(mountPoint, appImagePath) || TestAppImageSandbox(appImagePath)

	if needsNoSandbox {
		execCommand = fmt.Sprintf(`"%s" --no-sandbox`, appImagePath)
	} else {
		execCommand = fmt.Sprintf(`"%s"`, appImagePath)
	}

	version, icon, categories := ExtractMetadata(mountPoint)

	rawName := strings.TrimSuffix(filepath.Base(appImagePath), filepath.Ext(appImagePath))
	// Strip case-insensitive extension
	for _, ext := range []string{".AppImage", ".appimage", ".APPIMAGE"} {
		rawName = strings.TrimSuffix(rawName, ext)
	}
	appName := CleanAppImageName(rawName)
	appName = PromptForName(appName)

	iconPath := ""
	if icon != "" {
		ip, err := CopyIcon(icon, iconsDir())
		if err == nil {
			iconPath = ip
		}
	}

	if categories == "" {
		categories = "Utility;Application;"
	}

	_, err = CreateDesktopEntryAtomic(appName, execCommand, iconPath, version, categories)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: Failed to create desktop file: %v\n", err)
		return err
	}

	fmt.Printf("✓ Integrated %s successfully!\n", appName)
	return nil
}

func cmdList() {
	fmt.Println("Integrated AppImages")
	fmt.Println("===================")
	fmt.Println()

	entries, _ := ListAppImageDesktopEntries(applicationsDir())
	if len(entries) == 0 {
		fmt.Println("No AppImages integrated yet.")
		fmt.Println()
		fmt.Println("Tips:")
		fmt.Println("  - Run 'AppImageXdg find' to search for AppImages")
		fmt.Println("  - Run 'AppImageXdg install <file.AppImage>' to integrate an AppImage")
		fmt.Println("  - Run 'AppImageXdg help' for all commands")
		return
	}

	for i, e := range entries {
		execPath := ExecLineToPath(e.Exec)

		fmt.Printf("%d. %s\n", i+1, e.Name)
		v := e.Version
		if v == "" {
			v = "unknown"
		}
		fmt.Printf("   Version: %s\n", v)
		fmt.Printf("   Location: %s\n", execPath)
		fmt.Printf("   Desktop file: %s\n", e.Path)

		if _, err := os.Stat(execPath); os.IsNotExist(err) {
			fmt.Println("   ⚠️  WARNING: AppImage file missing!")
		}

		if strings.Contains(func() string { d, _ := os.ReadFile(e.Path); return string(d) }(), "Generated by AppImageXdg") {
			fmt.Println("   ✓ Managed by AppImageXdg")
		} else {
			fmt.Println("   ℹ️  Not managed by AppImageXdg (can still remove/update)")
		}
		fmt.Println()
	}
}

func cmdRemove(args []string) {
	if len(args) == 0 {
		fmt.Println("Usage: AppImageXdg remove <AppName>")
		fmt.Println()
		fmt.Println("Available AppImages:")

		entries, _ := ListAppImageDesktopEntries(applicationsDir())
		for _, e := range entries {
			fmt.Printf("  - %s\n", e.Name)
		}
		return
	}

	searchTerm := args[0]
	matches := FindDesktopEntries(applicationsDir(), searchTerm)

	switch len(matches) {
	case 0:
		fmt.Printf("No AppImage found matching: %s\n", searchTerm)
		fmt.Println()
		fmt.Println("Did you mean one of these?")
		allEntries, _ := ListAppImageDesktopEntries(applicationsDir())
		for _, e := range allEntries {
			fmt.Printf("  - %s\n", e.Name)
		}
	case 1:
		removeAppImageIntegration(matches[0])
	default:
		fmt.Printf("Multiple AppImages match '%s':\n", searchTerm)
		fmt.Println()
		for i, m := range matches {
			fmt.Printf("  %d) %s\n", i+1, m.Name)
		}
		fmt.Println("  0) Cancel")
		fmt.Println()
		fmt.Printf("Which one would you like to remove? (1-%d, 0 to cancel): ", len(matches))

		reader := bufio.NewReader(os.Stdin)
		choiceStr, _ := reader.ReadString('\n')
		choiceStr = strings.TrimSpace(choiceStr)

		var choice int
		fmt.Sscanf(choiceStr, "%d", &choice)

		if choice >= 1 && choice <= len(matches) {
			removeAppImageIntegration(matches[choice-1])
		} else if choice == 0 {
			fmt.Println("Removal cancelled.")
		} else {
			fmt.Println("Invalid choice. Removal cancelled.")
		}
	}
}

func removeAppImageIntegration(entry DesktopEntry) {
	fmt.Printf("Found: %s\n", entry.Name)
	fmt.Print("Remove this AppImage integration? (y/n): ")

	reader := bufio.NewReader(os.Stdin)
	response, _ := reader.ReadString('\n')
	response = strings.TrimSpace(strings.ToLower(response))

	if response == "y" || response == "yes" {
		if err := RemoveDesktopEntry(entry); err != nil {
			fmt.Fprintf(os.Stderr, "Error removing entry: %v\n", err)
		} else {
			fmt.Printf("✓ Removed %s integration\n", entry.Name)
		}
	} else {
		fmt.Println("Removal cancelled.")
	}
}

func cmdRun(args []string) {
	if len(args) == 0 {
		fmt.Println("Usage: AppImageXdg run <AppName>")
		fmt.Println()
		fmt.Println("Available AppImages:")
		entries, _ := ListAppImageDesktopEntries(applicationsDir())
		for _, e := range entries {
			fmt.Printf("  - %s\n", e.Name)
		}
		return
	}

	searchTerm := args[0]
	entries := FindDesktopEntries(applicationsDir(), searchTerm)

	for _, e := range entries {
		if strings.Contains(strings.ToLower(e.Name), strings.ToLower(searchTerm)) {
			execPath := ExecLineToPath(e.Exec)

			fmt.Printf("Running %s...\n", e.Name)
			fmt.Println("Press Ctrl+C to stop")
			fmt.Println(strings.Repeat("=", 40))

			parts := strings.Fields(execPath)
			if len(parts) == 0 {
				fmt.Fprintf(os.Stderr, "No executable path found\n")
				return
			}

			cmd := exec.Command(parts[0], parts[1:]...)
			cmd.Stdin = os.Stdin
			cmd.Stdout = os.Stdout
			cmd.Stderr = os.Stderr
			_ = cmd.Run()
			return
		}
	}

	fmt.Printf("No AppImage found matching: %s\n", searchTerm)
}

func cmdDebug(args []string) {
	if len(args) == 0 {
		fmt.Println("Usage: AppImageXdg debug <AppName>")
		fmt.Println()
		fmt.Println("Available AppImages:")
		entries, _ := ListAppImageDesktopEntries(applicationsDir())
		for _, e := range entries {
			fmt.Printf("  - %s\n", e.Name)
		}
		return
	}

	searchTerm := args[0]
	entries := FindDesktopEntries(applicationsDir(), searchTerm)

	for _, e := range entries {
		if strings.Contains(strings.ToLower(e.Name), strings.ToLower(searchTerm)) {
			execPath := ExecLineToPath(e.Exec)

			fmt.Printf("=== Debug Mode for %s ===\n", e.Name)
			fmt.Printf("AppImage: %s\n", execPath)
			fmt.Println()
			fmt.Println("Environment variables that affect AppImages:")
			fmt.Println("  APPIMAGE_EXTRACT_AND_RUN=1  - Extract and run (for FUSE issues)")
			fmt.Println("  APPDIR                      - AppImage mount directory")
			fmt.Println("  APPIMAGE                    - Path to the AppImage")
			fmt.Println("  APPIMAGE_DEBUG=1            - Enable debug logging in wrapper")
			fmt.Println("  APPIMAGE_VERBOSE=1          - Enable verbose output")
			fmt.Println()
			fmt.Printf("You can also run with environment variables:\n")
			fmt.Printf("  APPIMAGE_DEBUG=1 AppImageXdg run %s\n", searchTerm)
			fmt.Println()
			fmt.Println("Running with verbose output...")
			fmt.Println("Press Ctrl+C to stop")
			fmt.Println(strings.Repeat("=", 40))

			nameLower := strings.ToLower(e.Name)
			var debugFlags []string

			// Electron apps
			electronPatterns := []string{"via", "vscode", "discord", "slack", "teams", "obsidian", "element"}
			for _, p := range electronPatterns {
				if strings.Contains(nameLower, p) {
					debugFlags = append(debugFlags, "--verbose", "--enable-logging", "--log-level=verbose")
					fmt.Printf("Detected Electron app, using flags: %v\n", debugFlags)
					break
				}
			}

			// Qt/KDE
			if strings.Contains(nameLower, "qt") || strings.Contains(nameLower, "kde") {
				os.Setenv("QT_LOGGING_RULES", "*=true")
				fmt.Println("Enabled Qt verbose logging")
			}

			// GTK
			if strings.Contains(nameLower, "gtk") || strings.Contains(nameLower, "gnome") {
				os.Setenv("GTK_DEBUG", "all")
				fmt.Println("Enabled GTK debug output")
			}

			fmt.Print("Run with strace for system call tracing? (y/n): ")
			reader := bufio.NewReader(os.Stdin)
			useStrace, _ := reader.ReadString('\n')
			useStrace = strings.TrimSpace(strings.ToLower(useStrace))

			if useStrace == "y" || useStrace == "yes" {
				if _, err := exec.LookPath("strace"); err == nil {
					fmt.Println("Running with strace...")
					fmt.Println("Note: strace may cause FUSE mount issues with AppImages.")
					fmt.Println("If you see 'Cannot mount AppImage', try running without strace.")
					fmt.Println()

					straceArgs := []string{"-e", "trace=open,openat,access,stat,execve", "-f", execPath}
					straceArgs = append(straceArgs, debugFlags...)

					cmd := exec.Command("strace", straceArgs...)
					cmd.Env = append(os.Environ(),
						"APPIMAGE_EXTRACT_AND_RUN=1",
					)
					cmd.Stdin = os.Stdin
					cmd.Stdout = os.Stdout
					cmd.Stderr = os.Stderr
					_ = cmd.Run()
				} else {
					fmt.Println("strace not installed. Install with: sudo apt install strace")
					runAppImageDirect(execPath, debugFlags)
				}
			} else {
				runAppImageDirect(execPath, debugFlags)
			}
			return
		}
	}

	fmt.Printf("No AppImage found matching: %s\n", searchTerm)
}

func runAppImageDirect(path string, flags []string) {
	args := append([]string{path}, flags...)
	cmd := exec.Command(args[0], args[1:]...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	_ = cmd.Run()
}

func cmdDesktop() {
	fmt.Println("Desktop Files for AppImages")
	fmt.Println("===========================")
	fmt.Println()

	entries, _ := ListAppImageDesktopEntries(applicationsDir())
	if len(entries) == 0 {
		fmt.Println("No AppImage desktop files found.")
		return
	}

	for _, e := range entries {
		fmt.Printf("=== %s ===\n", e.Path)
		data, err := os.ReadFile(e.Path)
		if err == nil {
			fmt.Print(string(data))
		}
		fmt.Println()
	}
}
