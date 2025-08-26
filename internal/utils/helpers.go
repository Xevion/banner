package utils

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

// Options is a map of options from a discord command.
type Options map[string]*discordgo.ApplicationCommandInteractionDataOption

// GetInt returns the integer value of an option.
func (o Options) GetInt(key string) int64 {
	if opt, ok := o[key]; ok {
		return opt.IntValue()
	}
	return 0
}

// ParseOptions parses slash command options into a map.
func ParseOptions(options []*discordgo.ApplicationCommandInteractionDataOption) Options {
	optionMap := make(Options)
	for _, opt := range options {
		optionMap[opt.Name] = opt
	}
	return optionMap
}

// AddUserAgent adds a false but consistent user agent to the request
func AddUserAgent(req *http.Request) {
	req.Header.Add("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
}

// ContentTypeMatch checks if the Resty response has the given content type
func ContentTypeMatch(res *resty.Response, expectedContentType string) bool {
	contentType := res.Header().Get("Content-Type")
	if contentType == "" {
		return expectedContentType == "application/octect-stream"
	}
	return strings.HasPrefix(contentType, expectedContentType)
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

// Plural is a simple helper function that returns an empty string if n is 1, and "s" otherwise.
func Plural(n int) string {
	if n == 1 {
		return ""
	}
	return "s"
}

// Plurale is a simple helper function that returns an empty string if n is 1, and "ess" otherwise.
// This is for words that end in "es" when plural.
func Plurale(n int) string {
	if n == 1 {
		return ""
	}
	return "es"
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

// GetIntPointer returns a pointer to the given value.
// This function is useful for discordgo, which inexplicably requires pointers to integers for minLength arguments.
func GetIntPointer(value int) *int {
	return &value
}

// GetFloatPointer returns a pointer to the given value.
// This function is useful for discordgo, which inexplicably requires pointers to floats for minLength arguments.
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

func GuessExtension(contentType string) string {
	ext, ok := extensionMap[strings.ToLower(contentType)]
	if !ok {
		return ""
	}
	return ext
}

// DumpResponse dumps a Resty response body to a file for debugging purposes
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

func GetFetchedFooter(cfg *config.Config, time time.Time) *discordgo.MessageEmbedFooter {
	return &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Fetched at %s", time.In(cfg.CentralTimeLocation).Format("Monday, January 2, 2006 at 3:04:05PM")),
	}
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

// Point represents a point in 2D space
type Point struct {
	X, Y float64
}

func Slope(p1 Point, p2 Point, x float64) Point {
	slope := (p2.Y - p1.Y) / (p2.X - p1.X)
	newY := slope*(x-p1.X) + p1.Y
	return Point{X: x, Y: newY}
}
