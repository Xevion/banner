package main

import (
	"flag"
	"fmt"
	"net/http"
	"net/http/cookiejar"
	"os"
	"os/signal"
	"runtime"
	"strings"

	"github.com/bwmarrin/discordgo"
	"github.com/joho/godotenv"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/samber/lo"
)

var (
	// Base URL for all requests to the banner system
	baseURL               string
	client                http.Client
	cookies               http.CookieJar
	session               *discordgo.Session
	RemoveCommands        = flag.Bool("rmcmd", true, "Remove all commands after shutdowning or not")
	integerOptionMinValue = 0.0
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

func init() {
	env := os.Getenv("ENVIRONMENT")
	if env == "" {
		env = os.Getenv("RAILWAY_ENVIRONMENT")
		if env == "" {
			env = "development"
		}
	}
	var isDevelopment bool = env == "development"

	if isDevelopment {
		log.Logger = zerolog.New(logOut{}).With().Timestamp().Logger()
	}

	discordgo.Logger = func(msgL, caller int, format string, a ...interface{}) {
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
}

func main() {
	// Load environment variables
	if err := godotenv.Load(); err != nil {
		log.Debug().Err(err).Msg("Error loading .env file")
	}
	baseURL = os.Getenv("BANNER_BASE_URL")

	// Create cookie jar
	var err error
	cookies, err = cookiejar.New(nil)
	if err != nil {
		log.Err(err).Msg("Cannot create cookie jar")
	}

	// Create client, setup session (acquire cookies)
	client = http.Client{Jar: cookies}
	setup()

	// Create discord session
	session, err = discordgo.New("Bot " + os.Getenv("BOT_TOKEN"))
	if err != nil {
		log.Err(err).Msg("Invalid bot parameters")
	}

	// Open discord session
	session.AddHandler(func(s *discordgo.Session, r *discordgo.Ready) {
		log.Info().Str("username", r.User.Username).Str("discriminator", r.User.Discriminator).Str("id", r.User.ID).Str("session", s.State.SessionID).Msg("Bot is logged in")
	})
	err = session.Open()
	if err != nil {
		log.Fatal().Msgf("Cannot open the session: %v", err)
	}

	// Setup command handlers
	session.AddHandler(func(internalSession *discordgo.Session, interaction *discordgo.InteractionCreate) {
		if handler, ok := commandHandlers[interaction.ApplicationCommandData().Name]; ok {
			handler(internalSession, interaction)
		} else {
			log.Warn().Msgf("Unknown command '%v'", interaction.ApplicationCommandData().Name)
		}
	})

	// Register commands with discord
	arr := zerolog.Arr()
	lo.ForEach(commandDefinitions, func(cmd *discordgo.ApplicationCommand, _ int) {
		arr.Str(cmd.Name)
	})
	log.Info().Array("commands", arr).Msg("Registering commands")

	// Register commands
	registeredCommands := make([]*discordgo.ApplicationCommand, len(commandDefinitions))
	for i, cmdDefinition := range commandDefinitions {
		cmdInstance, err := session.ApplicationCommandCreate(session.State.User.ID, os.Getenv("BOT_TARGET_GUILD"), cmdDefinition)
		if err != nil {
			log.Panic().Err(err).Str("name", cmdDefinition.Name).Msgf("Cannot register command")
		}
		registeredCommands[i] = cmdInstance
	}

	// Cloes session, ensure
	defer session.Close()
	defer client.CloseIdleConnections()

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt)
	log.Info().Msg("Press Ctrl+C to exit")
	<-stop

	if *RemoveCommands {
		// log.Info().Array("commandIds", registeredCommands).Msg("Removing commands")

		for _, cmd := range registeredCommands {
			err := session.ApplicationCommandDelete(session.State.User.ID, os.Getenv("BOT_TARGET_GUILD"), cmd.ID)
			if err != nil {
				log.Err(err).Str("command", cmd.Name)
			}
		}
	}

	log.Warn().Msg("Gracefully shutting down")

}
