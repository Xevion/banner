package utils

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
type LogSplitter struct {
	Std io.Writer
	Err io.Writer
}

// Write should not be called
func (l LogSplitter) Write(p []byte) (n int, err error) {
	return l.Std.Write(p)
}

// WriteLevel write to the appropriate output
func (l LogSplitter) WriteLevel(level zerolog.Level, p []byte) (n int, err error) {
	if level <= zerolog.WarnLevel {
		return l.Std.Write(p)
	} else {
		return l.Err.Write(p)
	}
}
