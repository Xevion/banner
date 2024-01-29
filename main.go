package main

import (
	"context"
	"fmt"
	"net/http"
	"net/http/cookiejar"
	"os"
	"os/signal"
	"syscall"
	"time"
	_ "time/tzdata"

	"github.com/bwmarrin/discordgo"
	"github.com/joho/godotenv"
	"github.com/redis/go-redis/v9"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/samber/lo"
)

var (
	ctx                 context.Context
	kv                  *redis.Client
	session             *discordgo.Session
	client              http.Client
	cookies             http.CookieJar
	isDevelopment       bool
	baseURL             string // Base URL for all requests to the banner system
	environment         string
	CentralTimeLocation *time.Location
)

const (
	ICalTimestampFormatUtc   = "20060102T150405Z"
	ICalTimestampFormatLocal = "20060102T150405"
	CentralTimezoneName      = "America/Chicago"
)

func init() {
	// Load environment variables
	if err := godotenv.Load(); err != nil {
		log.Debug().Err(err).Msg("Error loading .env file")
	}

	ctx = context.Background()

	var err error
	CentralTimeLocation, err = time.LoadLocation(CentralTimezoneName)
	if err != nil {
		panic(err)
	}

	// Set zerolog's timestamp function to use the central timezone
	zerolog.TimestampFunc = func() time.Time {
		return time.Now().In(CentralTimeLocation)
	}

	// Try to grab the environment variable, or default to development
	environment = GetFirstEnv("ENVIRONMENT", "RAILWAY_ENVIRONMENT")
	if environment == "" {
		environment = "development"
	}

	// Use the custom console writer if we're in development
	isDevelopment = environment == "development"
	if isDevelopment {
		log.Logger = zerolog.New(logSplitter{std: stdConsole, err: errConsole}).With().Timestamp().Logger()
	} else {
		log.Logger = zerolog.New(logSplitter{std: os.Stdout, err: os.Stderr}).With().Timestamp().Logger()
	}
	log.Debug().Str("environment", environment).Msg("Loggers Setup")

	// Set discordgo's logger to use zerolog
	discordgo.Logger = DiscordGoLogger

	baseURL = os.Getenv("BANNER_BASE_URL")
}

func main() {
	// Setup redis
	redisUrl := GetFirstEnv("REDIS_URL", "REDIS_PRIVATE_URL")
	if redisUrl == "" {
		log.Fatal().Msg("REDIS_URL/REDIS_PRIVATE_URL not set")
	}

	// Parse URL and create client
	options, err := redis.ParseURL(redisUrl)
	if err != nil {
		log.Fatal().Err(err).Msg("Cannot parse redis url")
	}
	kv = redis.NewClient(options)

	var lastPingErr error
	pingCount := 0  // Nth ping being attempted
	totalPings := 5 // Total pings to attempt

	// Wait for private networking to kick in (production only)
	if !isDevelopment {
		time.Sleep(250 * time.Millisecond)
	}

	// Test the redis instance, try to ping every 2 seconds 5 times, otherwise panic
	for {
		pingCount++
		if pingCount > totalPings {
			log.Fatal().Err(lastPingErr).Msg("Reached ping limit while trying to connect")
		}

		// Ping redis
		pong, err := kv.Ping(ctx).Result()

		// Failed; log error and wait 2 seconds
		if err != nil {
			lastPingErr = err
			log.Warn().Err(err).Int("pings", pingCount).Int("remaining", totalPings-pingCount).Msg("Cannot ping redis")
			time.Sleep(2 * time.Second)

			continue
		}

		log.Debug().Str("ping", pong).Msg("Redis connection successful")
		break
	}

	// Create cookie jar
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
		log.Fatal().Err(err).Msg("Cannot open the session")
	}

	// Setup command handlers
	session.AddHandler(func(internalSession *discordgo.Session, interaction *discordgo.InteractionCreate) {
		name := interaction.ApplicationCommandData().Name
		if handler, ok := commandHandlers[name]; ok {
			// Build dict of options for the log
			options := zerolog.Dict()
			for _, option := range interaction.ApplicationCommandData().Options {
				options.Str(option.Name, fmt.Sprintf("%v", option.Value))
			}

			event := log.Info().Str("name", name).Str("user", GetUsername(interaction)).Dict("options", options)

			// If the command was invoked in a guild, add guild & channel info to the log
			if interaction.Member != nil {
				guild := zerolog.Dict()
				guild.Str("id", interaction.GuildID)
				guild.Str("name", GetGuildName(interaction.GuildID))
				event.Dict("guild", guild)

				channel := zerolog.Dict()
				channel.Str("id", interaction.ChannelID)
				guild.Str("name", GetChannelName(interaction.ChannelID))
				event.Dict("channel", channel)
			} else {
				// If the command was invoked in a DM, add the user info to the log
				user := zerolog.Dict()
				user.Str("id", interaction.User.ID)
				user.Str("name", interaction.User.Username)
				event.Dict("user", user)
			}

			// Log command invocation
			event.Msg("Command Invoked")

			// Call handler
			err := handler(internalSession, interaction)

			// Log & respond error
			if err != nil {
				// TODO: Find a way to merge the response with the handler's error
				log.Error().Str("commandName", name).Err(err).Msg("Command Handler Error")

				// Respond with error
				err = RespondError(internalSession, interaction.Interaction, fmt.Sprintf("Unexpected Error: %s", err.Error()), nil)
				if err != nil {
					log.Error().Str("commandName", name).Err(err).Msg("Failed to respond with error feedback")
				}
			}

		} else {
			log.Error().Str("commandName", name).Msg("Command Interaction Has No Handler")

			// Respond with error
			RespondError(internalSession, interaction.Interaction, "Unexpected Error: interaction has no handler", nil)
		}
	})

	// Register commands with discord
	arr := zerolog.Arr()
	lo.ForEach(commandDefinitions, func(cmd *discordgo.ApplicationCommand, _ int) {
		arr.Str(cmd.Name)
	})
	log.Info().Array("commands", arr).Msg("Registering commands")

	// In development, use test server, otherwise empty (global) for command registration
	guildTarget := ""
	if isDevelopment {
		guildTarget = os.Getenv("BOT_TARGET_GUILD")
	}

	// Register commands
	existingCommands, err := session.ApplicationCommands(session.State.User.ID, guildTarget)
	if err != nil {
		log.Fatal().Err(err).Msg("Cannot get existing commands")
	}
	newCommands, err := session.ApplicationCommandBulkOverwrite(session.State.User.ID, guildTarget, commandDefinitions)
	if err != nil {
		log.Fatal().Err(err).Msg("Cannot register commands")
	}

	// Compare existing commands with new commands
	for _, newCommand := range newCommands {
		existingCommand, found := lo.Find(existingCommands, func(cmd *discordgo.ApplicationCommand) bool {
			return cmd.Name == newCommand.Name
		})

		// New command
		if !found {
			log.Info().Str("commandName", newCommand.Name).Msg("Registered new command")
			continue
		}

		// Compare versions
		if newCommand.Version != existingCommand.Version {
			log.Info().Str("commandName", newCommand.Name).
				Str("oldVersion", existingCommand.Version).Str("newVersion", newCommand.Version).
				Msg("Command Updated")
		}
	}

	// Fetch terms on startup
	_, err = GetTerms("", 1, 10)
	if err != nil {
		log.Fatal().Err(err).Msg("Cannot get terms")
	}

	// Term Select Pre-Search POST
	var termSelect string
	currentTerm, nextTerm := GetCurrentTerm(time.Now())
	if currentTerm == nil {
		termSelect = nextTerm.ToString()
	} else {
		termSelect = currentTerm.ToString()
	}
	log.Info().Str("term", termSelect).Str("sessionID", sessionID).Msg("Setting selected term")
	SelectTerm(termSelect)

	// Close session, ensure http client closes idle connections
	defer session.Close()
	defer client.CloseIdleConnections()

	// Setup signal handler channel
	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt)    // Ctrl+C signal
	signal.Notify(stop, syscall.SIGTERM) // Container stop signal

	// Wait for signal (indefinite)
	<-stop

	// Defers are called after this
	log.Warn().Msg("Gracefully shutting down")
}
