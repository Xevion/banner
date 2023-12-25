package main

import (
	"os"

	"github.com/rs/zerolog"
)

// logOut implements zerolog.LevelWriter
type logOut struct{}

// Write should not be called
func (l logOut) Write(p []byte) (n int, err error) {
	return os.Stdout.Write(p)
}

const timeFormat = "2006-01-02 15:04:05"

var (
	standardOut = zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: timeFormat}
	errorOut    = zerolog.ConsoleWriter{Out: os.Stderr, TimeFormat: timeFormat}
)

// WriteLevel write to the appropriate output
func (l logOut) WriteLevel(level zerolog.Level, p []byte) (n int, err error) {
	if level <= zerolog.WarnLevel {
		return standardOut.Write(p)
	} else {
		return errorOut.Write(p)
	}
}
