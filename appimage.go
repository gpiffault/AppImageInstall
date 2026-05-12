package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"syscall"
	"time"
)

type AppImageMeta struct {
	Name       string
	Version    string
	Icon       string
	Categories string
	Path       string
}

func IsExecutable(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}
	return info.Mode()&0111 != 0
}

func IsElectronApp(mountPoint, name string) bool {
	if _, err := os.Stat(filepath.Join(mountPoint, "resources", "electron.asar")); err == nil {
		return true
	}
	if _, err := os.Stat(filepath.Join(mountPoint, "chrome-sandbox")); err == nil {
		return true
	}

	lower := strings.ToLower(name)
	for _, p := range []string{"via", "vscode", "discord", "slack", "teams", "obsidian", "element"} {
		if strings.Contains(lower, p) {
			return true
		}
	}
	return false
}

func MountAppImage(path string) (mountPoint string, pid int, err error) {
	if !IsExecutable(path) {
		if e := os.Chmod(path, 0755); e != nil {
			return "", 0, fmt.Errorf("cannot make executable: %w", e)
		}
	}

	cmd := exec.Command(path, "--appimage-mount")
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return "", 0, fmt.Errorf("stdout pipe: %w", err)
	}
	cmd.Stderr = nil

	if err := cmd.Start(); err != nil {
		return "", 0, fmt.Errorf("start mount: %w", err)
	}

	time.Sleep(1 * time.Second)

	pid = cmd.Process.Pid
	reader := bufio.NewReader(stdout)
	line, err := reader.ReadString('\n')
	if err != nil {
		cmd.Process.Kill()
		return "", 0, fmt.Errorf("read mount point: %w", err)
	}

	mountPoint = strings.TrimSpace(line)
	if mountPoint == "" {
		cmd.Process.Kill()
		return "", 0, fmt.Errorf("empty mount point")
	}

	return mountPoint, pid, nil
}

func UnmountAppImage(pid int, mountPoint string) {
	if pid > 0 {
		proc, err := os.FindProcess(pid)
		if err == nil {
			proc.Signal(syscall.SIGKILL)
		}
	}

	time.Sleep(500 * time.Millisecond)

	if mountPoint != "" {
		for _, cmd := range []string{"fusermount", "fusermount3"} {
			exec.Command(cmd, "-u", mountPoint).Run()
		}
	}
}

func ExtractMetadata(mountPoint string) (name, version, icon, categories string) {
	desktopFiles, _ := filepath.Glob(filepath.Join(mountPoint, "*.desktop"))

	if len(desktopFiles) > 0 {
		f, err := os.Open(desktopFiles[0])
		if err == nil {
			defer f.Close()
			scanner := bufio.NewScanner(f)
			for scanner.Scan() {
				line := scanner.Text()
				if name == "" && strings.HasPrefix(line, "Name=") {
					name = strings.TrimPrefix(line, "Name=")
				}
				if version == "" && strings.HasPrefix(line, "Version=") {
					version = strings.TrimPrefix(line, "Version=")
				}
				if categories == "" && strings.HasPrefix(line, "Categories=") {
					categories = strings.TrimPrefix(line, "Categories=")
				}
				if version != "" && categories != "" {
					break
				}
			}
		}
	}

	icon = FindIcon(mountPoint)
	if categories == "" {
		categories = "Utility;Application;"
	}

	return
}

func FindIcon(mountPoint string) string {
	patterns := []string{
		filepath.Join(mountPoint, "*256*icon*.png"),
		filepath.Join(mountPoint, "*128*icon*.png"),
		filepath.Join(mountPoint, "*64*icon*.png"),
		filepath.Join(mountPoint, "*48*icon*.png"),
		filepath.Join(mountPoint, "*icon*.png"),
		filepath.Join(mountPoint, "*icon*.svg"),
		filepath.Join(mountPoint, "*256*.png"),
		filepath.Join(mountPoint, "*128*.png"),
		filepath.Join(mountPoint, "*.png"),
		filepath.Join(mountPoint, "*.svg"),
	}

	for _, pattern := range patterns {
		matches, _ := filepath.Glob(pattern)
		if len(matches) > 0 {
			return matches[0]
		}
	}

	allIcons, _ := filepath.Glob(filepath.Join(mountPoint, "*.png"))
	if len(allIcons) > 0 {
		return allIcons[0]
	}
	allSvgs, _ := filepath.Glob(filepath.Join(mountPoint, "*.svg"))
	if len(allSvgs) > 0 {
		return allSvgs[0]
	}

	return ""
}

func CopyIcon(srcFile, dstDir string) (string, error) {
	if srcFile == "" {
		return "", nil
	}

	if err := os.MkdirAll(dstDir, 0755); err != nil {
		return "", err
	}

	src, err := os.Open(srcFile)
	if err != nil {
		return "", err
	}
	defer src.Close()

	base := filepath.Base(srcFile)
	dstPath := filepath.Join(dstDir, base)
	dst, err := os.Create(dstPath)
	if err != nil {
		return "", err
	}
	defer dst.Close()

	if _, err := dst.ReadFrom(src); err != nil {
		os.Remove(dstPath)
		return "", err
	}

	return dstPath, nil
}

func TestAppImageSandbox(path string) bool {
	cmd := exec.Command(path, "--help")
	cmd.Env = append(os.Environ(), "DISPLAY=")
	output, err := cmd.CombinedOutput()
	if err != nil {
		return false
	}
	return strings.Contains(string(output), "SUID sandbox helper binary")
}
