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
	ctx           context.Context
	kv            *redis.Client
	session       *discordgo.Session
	client        http.Client
	cookies       http.CookieJar
	isDevelopment bool
	baseURL       string // Base URL for all requests to the banner system
	environment   string
)

func init() {
	ctx = context.Background()

	// Try to grab the environment variable, or default to development
	environment := GetFirstEnv("ENVIRONMENT", "RAILWAY_ENVIRONMENT")
	if environment == "" {
		environment = "development"
	}

	// Use the custom console writer if we're in development
	isDevelopment = environment == "development"
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

		pong, err := kv.Ping(ctx).Result()
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
		// Create a hash of the command definition
		hash := xxhash.Sum64(structhash.Dump(cmdDefinition, 1))
		key := fmt.Sprintf("%s:command:%s:xxhash", environment, cmdDefinition.Name)

		// Get the stored hash
		storedHash, err := kv.Get(ctx, key).Uint64()
		if err != nil {
			if err != redis.Nil {
				log.Err(err).Msg("Cannot get command hash from redis")
			} else {
				log.Debug().Str("command", cmdDefinition.Name).Str("key", key).Msg("Command hash not found in redis")
			}
		}

		// If the hash is the same, skip registering the command
		if hash == storedHash {
			log.Debug().Str("command", cmdDefinition.Name).Str("key", key).Msg("Command hash matches, skipping registration")
			continue
		}

		// Unregister the old command
		oldCommandId, err := kv.Get(ctx, fmt.Sprintf("%s:command:%s:id", environment, cmdDefinition.Name)).Result()
		if err != nil {
			if err != redis.Nil {
				// It's an unlikely case, but if required, we could get the old command ID from discord to unregister it.
				log.Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot get old command ID from redis")
			} else {
				log.Debug().Str("name", cmdDefinition.Name).Str("key", key).Msg("Old command ID not found in redis")
			}
		} else {
			err = session.ApplicationCommandDelete(session.State.User.ID, "", oldCommandId)
			if err != nil {
				log.Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot unregister old command")
			} else {
				log.Info().Str("name", cmdDefinition.Name).Str("key", key).Msg("Unregistered old command")
			}
		}

		// Register the command
		cmdInstance, err := session.ApplicationCommandCreate(session.State.User.ID, "", cmdDefinition)
		if err != nil {
			log.Panic().Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot register command")
		}
		registeredCommands[i] = cmdInstance
		log.Info().Str("name", cmdDefinition.Name).Str("key", key).Msg("Registered command")

		// Store the hash for the new registered command
		err = kv.Set(ctx, key, hash, 0).Err()
		if err != nil {
			// No panic - the command is still registered, hash is only to prevent unnecessary registrations
			log.Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot set command hash in redis")
			continue
		}

		// Store the command ID to unregister later
		err = kv.Set(ctx, fmt.Sprintf("%s:command:%s:id", environment, cmdDefinition.Name), cmdInstance.ID, 0).Err()
		if err != nil {
			// No
			log.Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot set command ID in redis")
		}
	}

	// Cloes session, ensure http client closes idle connections
	defer session.Close()
	defer client.CloseIdleConnections()

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt)    // Ctrl+C signal
	signal.Notify(stop, syscall.SIGTERM) // Container stop signal

	<-stop

	// Defers are called after this
	log.Warn().Msg("Gracefully shutting down")
}
