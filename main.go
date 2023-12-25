package main

import (
	"context"
	"encoding/base64"
	"flag"
	"fmt"
	"net/http"
	"net/http/cookiejar"
	"os"
	"os/signal"

	"github.com/bwmarrin/discordgo"
	"github.com/cespare/xxhash/v2"
	"github.com/cnf/structhash"
	"github.com/joho/godotenv"
	"github.com/redis/go-redis/v9"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/samber/lo"
)

var (
	// Base URL for all requests to the banner system
	baseURL        string
	client         http.Client
	kv             *redis.Client
	ctx            context.Context
	cookies        http.CookieJar
	session        *discordgo.Session
	RemoveCommands = flag.Bool("rmcmd", true, "Remove all commands after shutdowning or not")
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
	ctx = context.Background()

	// Try to grab the environment variable, or default to development
	env := os.Getenv("ENVIRONMENT")
	if env == "" {
		env = os.Getenv("RAILWAY_ENVIRONMENT")
		if env == "" {
			env = "development"
		}
	}

	// Use the custom console writer if we're in development
	var isDevelopment bool = env == "development"
	if isDevelopment {
		log.Logger = zerolog.New(logOut{}).With().Timestamp().Logger()
	}

	// Set discordgo's logger to use zerolog
	discordgo.Logger = DiscordGoLogger

	// Load environment variables
	if err := godotenv.Load(); err != nil {
		log.Debug().Err(err).Msg("Error loading .env file")
	}
	baseURL = os.Getenv("BANNER_BASE_URL")
}

func main() {
	// Setup redis
	options, err := redis.ParseURL(os.Getenv("REDIS_URL"))
	if err != nil {
		log.Fatal().Err(err).Msg("Cannot parse redis url")
	}
	kv = redis.NewClient(options)

	// Test the redis instance
	pong, err := kv.Ping(ctx).Result()
	if err != nil {
		log.Fatal().Err(err).Msg("Cannot connect to redis")
	}
	log.Debug().Str("ping", pong).Msg("Redis connection successful")

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
			handler(internalSession, interaction)
		} else {
			log.Warn().Str("commandName", name).Msg("Unknown command")
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
		// Compare the hash to the hash stored in redis
		hash := xxhash.Sum64(structhash.Dump(cmdDefinition, 1))
		key := fmt.Sprintf("command:%s:xxhash", cmdDefinition.Name)
		keyB64 := base64.StdEncoding.EncodeToString([]byte(key))
		storedHash, err := kv.Get(ctx, key).Uint64()
		if err != nil {
			if err != redis.Nil {
				log.Err(err).Msg("Cannot get command hash from redis")
			} else {
				log.Debug().Str("command", cmdDefinition.Name).Str("key", keyB64).Msg("Command hash not found in redis")
			}
		}

		// If the hash is the same, skip registering the command
		if hash == storedHash {
			log.Debug().Str("command", cmdDefinition.Name).Str("key", keyB64).Msg("Command hash matches, skipping registration")
			continue
		}

		// Register the command
		cmdInstance, err := session.ApplicationCommandCreate(session.State.User.ID, "", cmdDefinition)
		if err != nil {
			log.Panic().Err(err).Str("name", cmdDefinition.Name).Str("key", keyB64).Msg("Cannot register command")
		}
		err = kv.Set(ctx, key, hash, 0).Err()
		if err != nil {
			log.Err(err).Str("name", cmdDefinition.Name).Str("key", keyB64).Msg("Cannot set command hash in redis")
		}
		registeredCommands[i] = cmdInstance
		log.Info().Str("name", cmdDefinition.Name).Str("key", keyB64).Msg("Registered command")
	}

	// Cloes session, ensure http client closes idle connections
	defer session.Close()
	defer client.CloseIdleConnections()

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt)
	log.Info().Msg("Press Ctrl+C to exit")
	<-stop

	log.Warn().Msg("Gracefully shutting down")

}
