package main

import (
	"fmt"
	"io"
	"math/rand"
	"net/http"
	"os"
	"runtime"
	"strconv"
	"strings"
	"time"

	"github.com/rs/zerolog"
	log "github.com/rs/zerolog/log"
)

func BuildRequestWithBody(method string, path string, params map[string]string, body io.Reader) *http.Request {
	// Builds a URL for the given path and parameters
	url := baseURL + path

	if params != nil {
		takenFirst := false
		for key, value := range params {
			paramChar := "&"
			if !takenFirst {
				paramChar = "?"
				takenFirst = true
			}

			url += paramChar + key + "=" + value
		}
	}

	request, _ := http.NewRequest(method, url, body)
	AddUserAgent(request)
	return request
}

func BuildRequest(method string, path string, params map[string]string) *http.Request {
	return BuildRequestWithBody(method, path, params, nil)
}

func AddUserAgent(req *http.Request) {
	req.Header.Add("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
}

// ContentTypeMatch checks if the response has the given content type
func ContentTypeMatch(response *http.Response, search string) bool {
	// Split on commas, check if any of the types match
	for _, content_type := range strings.Split(response.Header.Get("Content-Type"), ";") {
		if content_type == search {
			return true
		}
	}
	return false
}

const letterBytes = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

// RandomString returns a random string of length n using the letterBytes constant
// The constant used is specifically chosen to mimic Ellucian's banner session ID generation.
func RandomString(n int) string {
	b := make([]byte, n)
	for i := range b {
		b[i] = letterBytes[rand.Intn(len(letterBytes))]
	}
	return string(b)
}

// DiscordGoLogger is a specialized helper function that implements discordgo's global logging interface.
// It directs all logs to the zerolog implementation.
func DiscordGoLogger(msgL, caller int, format string, a ...interface{}) {
	pc, file, line, _ := runtime.Caller(caller)

	files := strings.Split(file, "/")
	file = files[len(files)-1]

	name := runtime.FuncForPC(pc).Name()
	fns := strings.Split(name, ".")
	name = fns[len(fns)-1]

	msg := fmt.Sprintf(format, a...)

	var event *zerolog.Event
	switch msgL {
	case 0:
		event = log.Debug()
	case 1:
		event = log.Info()
	case 2:
		event = log.Warn()
	case 3:
		event = log.Error()
	default:
		event = log.Info()
	}

	event.Str("file", file).Int("line", line).Str("function", name).Msg(msg)
}

// Nonce returns a string made up of the current time in milliseconds, Unix epoch/UTC
func Nonce() string {
	return strconv.Itoa(int(time.Now().UnixMilli()))
}

// doRequest performs & logs the request, logging and returning the response
func doRequest(req *http.Request) (*http.Response, error) {
	log.Debug().Str("method", strings.TrimRight(req.Method, " ")).Str("url", req.URL.String()).Str("query", req.URL.RawQuery).Str("content-type", req.Header.Get("Content-Type")).Msg("Request")
	res, err := client.Do(req)
	if err != nil {
		log.Err(err).Str("method", req.Method).Msg("Request Failed")
	} else {
		log.Debug().Str("status", res.Status).Int64("content-length", res.ContentLength).Strs("content-type", res.Header["Content-Type"]).Msg("Response")
	}
	return res, err
}

func Plural(n int) string {
	if n == 1 {
		return ""
	}
	return "s"
}

func WeekdaysToString(days map[time.Weekday]bool) string {
	str := ""

	if days[time.Monday] {
		str += "M"
	}

	if days[time.Tuesday] {
		str += "Tu"
	}

	if days[time.Wednesday] {
		str += "W"
	}

	if days[time.Thursday] {
		str += "Th"
	}

	if days[time.Friday] {
		str += "F"
	}

	if days[time.Saturday] {
		str += "Sa"
	}

	if days[time.Sunday] {
		str += "Su"
	}

	return str
}

type NaiveTime struct {
	Hours   uint
	Minutes uint
}

func ParseNaiveTime(integer uint64) NaiveTime {
	minutes := uint(integer % 100)
	hours := uint(integer / 100)

	return NaiveTime{Hours: hours, Minutes: minutes}
}

func (nt NaiveTime) String() string {
	meridiem := "AM"
	hour := nt.Hours
	if nt.Hours >= 12 {
		meridiem = "PM"
		hour -= 12
	}
	return fmt.Sprintf("%d:%02d%s", hour, nt.Minutes, meridiem)
}

func GetFirstEnv(key ...string) string {
	for _, k := range key {
		if v := os.Getenv(k); v != "" {
			return v
		}
	}
	return ""
}
