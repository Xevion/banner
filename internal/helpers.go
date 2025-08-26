package internal

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

	"github.com/bwmarrin/discordgo"
	"github.com/rs/zerolog"
	log "github.com/rs/zerolog/log"
	"resty.dev/v3"

	"banner/internal/config"
)

// Options is a map of options from a Discord command.
type Options map[string]*discordgo.ApplicationCommandInteractionDataOption

// GetInt returns the integer value of an option, or 0 if it doesn't exist.
func (o Options) GetInt(key string) int64 {
	if opt, ok := o[key]; ok {
		return opt.IntValue()
	}
	return 0
}

// ParseOptions parses slash command options into a map for easier access.
func ParseOptions(options []*discordgo.ApplicationCommandInteractionDataOption) Options {
	optionMap := make(Options)
	for _, opt := range options {
		optionMap[opt.Name] = opt
	}
	return optionMap
}

// AddUserAgent adds a consistent user agent to the request to mimic a real browser.
func AddUserAgent(req *http.Request) {
	req.Header.Add("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
}

// ContentTypeMatch checks if a Resty response has the given content type.
func ContentTypeMatch(res *resty.Response, expectedContentType string) bool {
	contentType := res.Header().Get("Content-Type")
	if contentType == "" {
		return expectedContentType == "application/octect-stream"
	}
	return strings.HasPrefix(contentType, expectedContentType)
}

const letterBytes = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

// RandomString returns a random string of length n.
// The character set is chosen to mimic Ellucian's Banner session ID generation.
func RandomString(n int) string {
	b := make([]byte, n)
	for i := range b {
		b[i] = letterBytes[rand.Intn(len(letterBytes))]
	}
	return string(b)
}

// DiscordGoLogger is a helper function that implements discordgo's logging interface, directing all logs to zerolog.
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

// Nonce returns the current time in milliseconds since the Unix epoch as a string.
// This is typically used as a query parameter to prevent request caching.
func Nonce() string {
	return strconv.Itoa(int(time.Now().UnixMilli()))
}

// Plural returns "s" if n is not 1.
func Plural(n int) string {
	if n == 1 {
		return ""
	}
	return "s"
}

// Plurale returns "es" if n is not 1.
func Plurale(n int) string {
	if n == 1 {
		return ""
	}
	return "es"
}

// WeekdaysToString converts a map of weekdays to a compact string representation (e.g., "MWF").
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

// NaiveTime represents a time of day without a date or timezone.
type NaiveTime struct {
	Hours   uint
	Minutes uint
}

// Sub returns the duration between two NaiveTime instances.
func (nt *NaiveTime) Sub(other *NaiveTime) time.Duration {
	return time.Hour*time.Duration(nt.Hours-other.Hours) + time.Minute*time.Duration(nt.Minutes-other.Minutes)
}

// ParseNaiveTime converts an integer representation of time (e.g., 1430) to a NaiveTime struct.
func ParseNaiveTime(integer uint64) *NaiveTime {
	minutes := uint(integer % 100)
	hours := uint(integer / 100)

	return &NaiveTime{Hours: hours, Minutes: minutes}
}

// String returns a string representation of the NaiveTime in 12-hour format (e.g., "2:30PM").
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

// GetFirstEnv returns the value of the first environment variable that is set.
func GetFirstEnv(key ...string) string {
	for _, k := range key {
		if v := os.Getenv(k); v != "" {
			return v
		}
	}
	return ""
}

// GetIntPointer returns a pointer to the given integer.
func GetIntPointer(value int) *int {
	return &value
}

// GetFloatPointer returns a pointer to the given float.
func GetFloatPointer(value float64) *float64 {
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

// GuessExtension guesses the file extension for a given content type.
func GuessExtension(contentType string) string {
	ext, ok := extensionMap[strings.ToLower(contentType)]
	if !ok {
		return ""
	}
	return ext
}

// DumpResponse dumps the body of a Resty response to a file for debugging.
func DumpResponse(res *resty.Response) {
	contentType := res.Header().Get("Content-Type")
	ext := GuessExtension(contentType)

	// Use current time as filename + /dumps/ prefix
	filename := fmt.Sprintf("dumps/%d.%s", time.Now().Unix(), ext)
	file, err := os.Create(filename)

	if err != nil {
		log.Err(err).Stack().Msg("Error creating file")
		return
	}
	defer file.Close()

	body, err := io.ReadAll(res.Body)
	if err != nil {
		log.Err(err).Stack().Msg("Error reading response body")
		return
	}

	_, err = file.Write(body)
	if err != nil {
		log.Err(err).Stack().Msg("Error writing response body")
		return
	}

	log.Info().Str("filename", filename).Str("content-type", contentType).Msg("Dumped response body")
}

// RespondError responds to an interaction with a formatted error message.
func RespondError(session *discordgo.Session, interaction *discordgo.Interaction, message string, err error) error {
	// Optional: log the error
	if err != nil {
		log.Err(err).Stack().Msg(message)
	}

	return session.InteractionRespond(interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Footer: &discordgo.MessageEmbedFooter{
						Text: fmt.Sprintf("Occurred at %s", time.Now().Format("Monday, January 2, 2006 at 3:04:05PM")),
					},
					Description: message,
					Color:       0xff0000,
				},
			},
			AllowedMentions: &discordgo.MessageAllowedMentions{},
		},
	})
}

// GetFetchedFooter returns a standard footer for embeds, indicating when the data was fetched.
func GetFetchedFooter(cfg *config.Config, time time.Time) *discordgo.MessageEmbedFooter {
	return &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Fetched at %s", time.In(cfg.CentralTimeLocation).Format("Monday, January 2, 2006 at 3:04:05PM")),
	}
}

// GetUser returns the user from an interaction, regardless of whether it was in a guild or a DM.
func GetUser(interaction *discordgo.InteractionCreate) *discordgo.User {
	// If the interaction is in a guild, the user is in the Member field
	if interaction.Member != nil {
		return interaction.Member.User
	}

	// If the interaction is in a DM, the user is in the User field
	return interaction.User
}

// EncodeParams encodes a map of parameters into a URL-encoded string, sorted by key.
func EncodeParams(params map[string]*[]string) string {
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

		for _, v := range *values {
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

// Point represents a point in 2D space.
type Point struct {
	X, Y float64
}

// Slope calculates the y-coordinate of a point on a line given two other points and an x-coordinate.
func Slope(p1 Point, p2 Point, x float64) Point {
	slope := (p2.Y - p1.Y) / (p2.X - p1.X)
	newY := slope*(x-p1.X) + p1.Y
	return Point{X: x, Y: newY}
}
