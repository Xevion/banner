package main

import (
	"io"
	"os"

	"github.com/rs/zerolog"
)

const timeFormat = "2006-01-02 15:04:05"

var (
	stdConsole = zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: timeFormat}
	errConsole = zerolog.ConsoleWriter{Out: os.Stderr, TimeFormat: timeFormat}
)

// logSplitter implements zerolog.LevelWriter
type logSplitter struct {
	std io.Writer
	err io.Writer
}

// Write should not be called
func (l logSplitter) Write(p []byte) (n int, err error) {
	return l.std.Write(p)
}

// WriteLevel write to the appropriate output
func (l logSplitter) WriteLevel(level zerolog.Level, p []byte) (n int, err error) {
	if level <= zerolog.WarnLevel {
		return l.std.Write(p)
	} else {
		return l.err.Write(p)
	}
}
