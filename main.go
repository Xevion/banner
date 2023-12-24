package main

import (
	"flag"
	"net/http"
	"net/http/cookiejar"
	"os"
	"os/signal"

	"github.com/bwmarrin/discordgo"
	"github.com/joho/godotenv"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
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
	log.Logger = zerolog.New(logOut{}).With().Timestamp().Logger()
}

func main() {
	// Load environment variables
	if err := godotenv.Load(); err != nil {
		log.Debug().Err(err).Msg("Error loading .env file")
	}
	baseURL = os.Getenv("BANNER_BASE_URL")

	var err error
	cookies, err = cookiejar.New(nil)
	if err != nil {
		log.Err(err).Msg("Cannot create cookie jar")
	}

	client = http.Client{Jar: cookies}
	setup(&cookies)

	session, err = discordgo.New("Bot " + os.Getenv("BOT_TOKEN"))
	if err != nil {
		log.Err(err).Msg("Invalid bot parameters")
	}

	session.AddHandler(func(s *discordgo.Session, r *discordgo.Ready) {
		// log.WithFields(log.Fields{
		// 	"username":      r.User.Username,
		// 	"discriminator": r.User.Discriminator,
		// 	"id":            r.User.ID,
		// 	"session":       s.State.SessionID,
		// }).Info("Bot is logged in")
		// log.
	})
	err = session.Open()
	if err != nil {
		log.Fatal().Msgf("Cannot open the session: %v", err)
	}

	session.AddHandler(func(internalSession *discordgo.Session, interaction *discordgo.InteractionCreate) {
		if handler, ok := commandHandlers[interaction.ApplicationCommandData().Name]; ok {
			handler(internalSession, interaction)
		}
	})

	log.Printf("Adding %d command%s...", len(commandDefinitions), Plural(len(commandDefinitions)))
	registeredCommands := make([]*discordgo.ApplicationCommand, len(commandDefinitions))
	for i, v := range commandDefinitions {
		cmd, err := session.ApplicationCommandCreate(session.State.User.ID, os.Getenv("BOT_TARGET_GUILD"), v)
		if err != nil {
			log.Panic().Msgf("Cannot create '%v' command: %v", v.Name, err)
		}
		registeredCommands[i] = cmd
	}

	defer session.Close()

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt)
	log.Info().Msgf("Press Ctrl+C to exit")
	<-stop

	if *RemoveCommands {
		log.Printf("Removing %d command%s...\n", len(registeredCommands), Plural(len(registeredCommands)))

		for _, v := range registeredCommands {
			err := session.ApplicationCommandDelete(session.State.User.ID, os.Getenv("BOT_TARGET_GUILD"), v.ID)
			if err != nil {
				log.Error().Msgf("Cannot delete '%v' command: %v", v.Name, err)
			}
		}
	}

	log.Info().Msg("Gracefully shutting down.")

}
