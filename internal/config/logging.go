package config

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

// NewConsoleWriter creates a new console writer with improved formatting for development
func NewConsoleWriter() zerolog.LevelWriter {
	return &ConsoleLogSplitter{
		stdConsole: zerolog.ConsoleWriter{
			Out:           os.Stdout,
			TimeFormat:    timeFormat,
			NoColor:       false,
			PartsOrder:    []string{zerolog.TimestampFieldName, zerolog.LevelFieldName, zerolog.MessageFieldName},
			PartsExclude:  []string{},
			FieldsExclude: []string{},
		},
		errConsole: zerolog.ConsoleWriter{
			Out:           os.Stderr,
			TimeFormat:    timeFormat,
			NoColor:       false,
			PartsOrder:    []string{zerolog.TimestampFieldName, zerolog.LevelFieldName, zerolog.MessageFieldName},
			PartsExclude:  []string{},
			FieldsExclude: []string{},
		},
	}
}

// ConsoleLogSplitter implements zerolog.LevelWriter with console formatting
type ConsoleLogSplitter struct {
	stdConsole zerolog.ConsoleWriter
	errConsole zerolog.ConsoleWriter
}

// Write should not be called
func (c *ConsoleLogSplitter) Write(p []byte) (n int, err error) {
	return c.stdConsole.Write(p)
}

// WriteLevel write to the appropriate output with console formatting
func (c *ConsoleLogSplitter) WriteLevel(level zerolog.Level, p []byte) (n int, err error) {
	if level <= zerolog.WarnLevel {
		return c.stdConsole.Write(p)
	}
	return c.errConsole.Write(p)
}

// LogSplitter implements zerolog.LevelWriter
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
