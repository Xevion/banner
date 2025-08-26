// Package config provides the configuration and logging setup for the application.
package config

import (
	"io"
	"os"

	"github.com/rs/zerolog"
)

const timeFormat = "2006-01-02 15:04:05"

// NewConsoleWriter creates a new console writer that splits logs between stdout and stderr.
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

// ConsoleLogSplitter is a zerolog.LevelWriter that writes to stdout for info/debug logs and stderr for warn/error logs, with console-friendly formatting.
type ConsoleLogSplitter struct {
	stdConsole zerolog.ConsoleWriter
	errConsole zerolog.ConsoleWriter
}

// Write is a passthrough to the standard console writer and should not be called directly.
func (c *ConsoleLogSplitter) Write(p []byte) (n int, err error) {
	return c.stdConsole.Write(p)
}

// WriteLevel writes to the appropriate output (stdout or stderr) with console formatting based on the log level.
func (c *ConsoleLogSplitter) WriteLevel(level zerolog.Level, p []byte) (n int, err error) {
	if level <= zerolog.WarnLevel {
		return c.stdConsole.Write(p)
	}
	return c.errConsole.Write(p)
}

// LogSplitter is a zerolog.LevelWriter that writes to stdout for info/debug logs and stderr for warn/error logs.
type LogSplitter struct {
	Std io.Writer
	Err io.Writer
}

// Write is a passthrough to the standard writer and should not be called directly.
func (l LogSplitter) Write(p []byte) (n int, err error) {
	return l.Std.Write(p)
}

// WriteLevel writes to the appropriate output (stdout or stderr) based on the log level.
func (l LogSplitter) WriteLevel(level zerolog.Level, p []byte) (n int, err error) {
	if level <= zerolog.WarnLevel {
		return l.Std.Write(p)
	}
	return l.Err.Write(p)
}
