package main

import (
	"bytes"
	"os"

	"github.com/rs/zerolog"
)

const timeFormat = "2006-01-02 15:04:05"

var (
	standardOut = zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: timeFormat}
	errorOut    = zerolog.ConsoleWriter{Out: os.Stderr, TimeFormat: timeFormat}
	buffer      = new(bytes.Buffer)
	bufferOut   = zerolog.ConsoleWriter{Out: buffer, TimeFormat: ""}
)

// logSplitter implements zerolog.LevelWriter
type logSplitter struct{}

// Write should not be called
func (l logSplitter) Write(p []byte) (n int, err error) {
	return os.Stdout.Write(p)
}

// WriteLevel write to the appropriate output
func (l logSplitter) WriteLevel(level zerolog.Level, p []byte) (n int, err error) {
	if level <= zerolog.WarnLevel {
		return standardOut.Write(p)
	} else {
		return errorOut.Write(p)
	}
}

// JsonColorizerHook implements zerolog.Hook
// This hook is used to reformat the output of the JSON 'message' field to be like ConsoleWriter, but embedded within JSON
type JsonColorizerHook struct{}

func (h JsonColorizerHook) Run(e *zerolog.Event, level zerolog.Level, msg string) {
	buffer.Reset()
	bufferOut.Write([]byte(msg))
	formattedMessage := buffer.String()

	e.Str("text", msg) // Add the original message to the event
	e.Str("message", formattedMessage)
}
