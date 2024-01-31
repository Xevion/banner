package main

import (
	"fmt"
	"io"
	"math/rand"
	"net/http"
	"net/url"
	"os"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"time"

	ics "github.com/arran4/golang-ical"
	"github.com/bwmarrin/discordgo"
	"github.com/rs/zerolog"
	log "github.com/rs/zerolog/log"
)

// BuildRequestWithBody builds a request with the given method, path, parameters, and body
func BuildRequestWithBody(method string, path string, params map[string]string, body io.Reader) *http.Request {
	// Builds a URL for the given path and parameters
	requestUrl := baseURL + path

	if params != nil {
		takenFirst := false
		for key, value := range params {
			paramChar := "&"
			if !takenFirst {
				paramChar = "?"
				takenFirst = true
			}

			requestUrl += paramChar + url.QueryEscape(key) + "=" + url.QueryEscape(value)
		}
	}

	request, _ := http.NewRequest(method, requestUrl, body)
	AddUserAgent(request)
	return request
}

// BuildRequest builds a request with the given method, path, and parameters and an empty body
func BuildRequest(method string, path string, params map[string]string) *http.Request {
	return BuildRequestWithBody(method, path, params, nil)
}

// AddUserAgent adds a false but consistent user agent to the request
func AddUserAgent(req *http.Request) {
	req.Header.Add("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
}

// ContentTypeMatch checks if the response has the given content type
func ContentTypeMatch(response *http.Response, search string) bool {
	contentType := response.Header.Get("Content-Type")
	if contentType == "" {
		return search == "application/octect-stream"
	}

	return strings.HasPrefix(contentType, search)
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
// This is typically used as a query parameter to prevent request caching in the browser.
func Nonce() string {
	return strconv.Itoa(int(time.Now().UnixMilli()))
}

// DoRequest performs & logs the request, logging and returning the response
func DoRequest(req *http.Request) (*http.Response, error) {
	log.Debug().
		Str("method", strings.TrimRight(req.Method, " ")).
		Str("url", req.URL.String()).
		Str("query", req.URL.RawQuery).
		Str("content-type", req.Header.Get("Content-Type")).
		Msg("Request")

	res, err := client.Do(req)

	if err != nil {
		log.Err(err).Stack().Str("method", req.Method).Msg("Request Failed")
	} else {
		contentLengthHeader := res.Header.Get("Content-Length")
		contentLength := int64(-1)

		// Get the content length
		if contentLengthHeader != "" {
			contentLength, err = strconv.ParseInt(contentLengthHeader, 10, 64)
			if err != nil {
				contentLength = -1
			}
		}

		log.Debug().Int("status", res.StatusCode).Int64("content-length", contentLength).Strs("content-type", res.Header["Content-Type"]).Msg("Response")
	}
	return res, err
}

// Plural is a simple helper function that returns an empty string if n is 1, and "s" otherwise.
func Plural(n int) string {
	if n == 1 {
		return ""
	}
	return "s"
}

func WeekdaysToString(days map[time.Weekday]bool) string {
	// If no days are present
	numDays := len(days)
	if numDays == 0 {
		return "None"
	}

	// If all days are present
	if numDays == 7 {
		return "Everyday"
	}

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

func (nt *NaiveTime) Sub(other *NaiveTime) time.Duration {
	return time.Hour*time.Duration(nt.Hours-other.Hours) + time.Minute*time.Duration(nt.Minutes-other.Minutes)
}

func ParseNaiveTime(integer uint64) *NaiveTime {
	minutes := uint(integer % 100)
	hours := uint(integer / 100)

	return &NaiveTime{Hours: hours, Minutes: minutes}
}

func (nt NaiveTime) String() string {
	meridiem := "AM"
	hour := nt.Hours
	if nt.Hours >= 12 {
		meridiem = "PM"
		if nt.Hours > 12 {
			hour -= 12
		}
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

// GetPointer returns a pointer to the given value.
// This function is useful for discordgo, which inexplicably requires pointers to integers for minLength arguments.
func GetPointer(value int) *int {
	return &value
}

var extensionMap = map[string]string{
	"text/plain":               "txt",
	"application/json":         "json",
	"text/html":                "html",
	"text/css":                 "css",
	"text/csv":                 "csv",
	"text/calendar":            "ics",
	"text/markdown":            "md",
	"text/xml":                 "xml",
	"text/yaml":                "yaml",
	"text/javascript":          "js",
	"text/vtt":                 "vtt",
	"image/jpeg":               "jpg",
	"image/png":                "png",
	"image/gif":                "gif",
	"image/webp":               "webp",
	"image/tiff":               "tiff",
	"image/svg+xml":            "svg",
	"image/bmp":                "bmp",
	"image/vnd.microsoft.icon": "ico",
	"image/x-icon":             "ico",
	"image/x-xbitmap":          "xbm",
	"image/x-xpixmap":          "xpm",
	"image/x-xwindowdump":      "xwd",
	"image/avif":               "avif",
	"image/apng":               "apng",
	"image/jxl":                "jxl",
}

func GuessExtension(contentType string) string {
	ext, ok := extensionMap[strings.ToLower(contentType)]
	if !ok {
		return ""
	}
	return ext
}

// DumpResponse dumps a response body to a file for debugging purposes
func DumpResponse(res *http.Response) {
	contentType := res.Header.Get("Content-Type")
	ext := GuessExtension(contentType)

	// Use current time as filename + /dumps/ prefix
	filename := fmt.Sprintf("dumps/%d.%s", time.Now().Unix(), ext)
	file, err := os.Create(filename)

	if err != nil {
		log.Err(err).Stack().Msg("Error creating file")
		return
	}
	defer file.Close()

	_, err = io.Copy(file, res.Body)
	if err != nil {
		log.Err(err).Stack().Msg("Error copying response body")
		return
	}

	log.Info().Str("filename", filename).Str("content-type", contentType).Msg("Dumped response body")
}

// ResponseError responds to an interaction with an error message
// TODO: Improve with a proper embed and colors
func RespondError(session *discordgo.Session, interaction *discordgo.Interaction, message string, err error) error {
	// Optional: log the error
	if err != nil {
		log.Err(err).Stack().Msg(message)
	}

	return session.InteractionRespond(interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Content: message,
		},
	})

}

func GetFooter(time time.Time) *discordgo.MessageEmbedFooter {
	return &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Fetched at %s", time.Format("Monday, January 2, 2006 at 3:04:05PM")),
	}
}

func NewCalendar() *ics.Calendar {
	c := &ics.Calendar{
		Components:         []ics.Component{},
		CalendarProperties: []ics.CalendarProperty{},
	}
	c.SetVersion("2.0")
	c.SetProductId("-//xevion//Banner Discord Bot//EN")
	return c
}

// GetUser returns the user from the interaction.
// This helper method is useful as depending on where the message was sent (guild or DM), the user is in a different field.
func GetUser(interaction *discordgo.InteractionCreate) *discordgo.User {
	// If the interaction is in a guild, the user is kept in the Member field
	if interaction.Member != nil {
		return interaction.Member.User
	}

	// If the interaction is in a DM, the user is kept in the User field
	return interaction.User
}

// Encode encodes the values into “URL encoded” form
// ("bar=baz&foo=quux") sorted by key.
func EncodeParams(params map[string][]string) string {
	// Escape hatch for nil
	if params == nil {
		return ""
	}

	// Sort the keys
	keys := make([]string, 0, len(params))
	for k := range params {
		keys = append(keys, k)
	}
	sort.Strings(keys)

	var buf strings.Builder
	for _, k := range keys {
		// Multiple values are allowed, so extract the slice & prepare the key
		values := params[k]
		keyEscaped := url.QueryEscape(k)

		for _, v := range values {
			// If any parameters have been written, add the ampersand
			if buf.Len() > 0 {
				buf.WriteByte('&')
			}

			// Write the key and value
			buf.WriteString(keyEscaped)
			buf.WriteByte('=')
			buf.WriteString(url.QueryEscape(v))
		}
	}

	return buf.String()
}
