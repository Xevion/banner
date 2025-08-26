package main

import (
	"flag"
	"net/http"
	"net/http/cookiejar"
	_ "net/http/pprof"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"
	_ "time/tzdata"

	"github.com/bwmarrin/discordgo"
	"github.com/joho/godotenv"
	"github.com/redis/go-redis/v9"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/rs/zerolog/pkgerrors"
	"github.com/samber/lo"
	"golang.org/x/text/message"

	"banner/internal/api"
	"banner/internal/bot"
	"banner/internal/config"
	"banner/internal/utils"
)

var (
	Session *discordgo.Session
	p       *message.Printer = message.NewPrinter(message.MatchLanguage("en"))
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

	// Set zerolog's timestamp function to use the central timezone
	zerolog.TimestampFunc = func() time.Time {
		// TODO: Move this to config
		loc, err := time.LoadLocation(CentralTimezoneName)
		if err != nil {
			panic(err)
		}
		return time.Now().In(loc)
	}

	zerolog.ErrorStackMarshaler = pkgerrors.MarshalStack

	// Use the custom console writer if we're in development
	isDevelopment := utils.GetFirstEnv("ENVIRONMENT", "RAILWAY_ENVIRONMENT")
	if isDevelopment == "" {
		isDevelopment = "development"
	}

	if isDevelopment == "development" {
		log.Logger = zerolog.New(utils.LogSplitter{Std: os.Stdout, Err: os.Stderr}).With().Timestamp().Logger()
	} else {
		log.Logger = zerolog.New(utils.LogSplitter{Std: os.Stdout, Err: os.Stderr}).With().Timestamp().Logger()
	}
	log.Debug().Str("environment", isDevelopment).Msg("Loggers Setup")

	// Set discordgo's logger to use zerolog
	discordgo.Logger = utils.DiscordGoLogger
}

func initRedis(cfg *config.Config) {
	// Setup redis
	redisUrl := utils.GetFirstEnv("REDIS_URL", "REDIS_PRIVATE_URL")
	if redisUrl == "" {
		log.Fatal().Stack().Msg("REDIS_URL/REDIS_PRIVATE_URL not set")
	}

	// Parse URL and create client
	options, err := redis.ParseURL(redisUrl)
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Cannot parse redis url")
	}
	kv := redis.NewClient(options)
	cfg.SetRedis(kv)

	var lastPingErr error
	pingCount := 0  // Nth ping being attempted
	totalPings := 5 // Total pings to attempt

	// Wait for private networking to kick in (production only)
	if !cfg.IsDevelopment {
		time.Sleep(250 * time.Millisecond)
	}

	// Test the redis instance, try to ping every 2 seconds 5 times, otherwise panic
	for {
		pingCount++
		if pingCount > totalPings {
			log.Fatal().Stack().Err(lastPingErr).Msg("Reached ping limit while trying to connect")
		}

		// Ping redis
		pong, err := cfg.KV.Ping(cfg.Ctx).Result()

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
}

func main() {
	flag.Parse()

	cfg, err := config.New()
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Cannot create config")
	}

	// Try to grab the environment variable, or default to development
	environment := utils.GetFirstEnv("ENVIRONMENT", "RAILWAY_ENVIRONMENT")
	if environment == "" {
		environment = "development"
	}
	cfg.SetEnvironment(environment)

	initRedis(cfg)

	if strings.EqualFold(os.Getenv("PPROF_ENABLE"), "true") {
		// Start pprof server
		go func() {
			port := os.Getenv("PORT")
			log.Info().Str("port", port).Msg("Starting pprof server")
			err := http.ListenAndServe(":"+port, nil)

			if err != nil {
				log.Fatal().Stack().Err(err).Msg("Cannot start pprof server")
			}
		}()
	}

	// Create cookie jar
	cookies, err := cookiejar.New(nil)
	if err != nil {
		log.Err(err).Msg("Cannot create cookie jar")
	}

	// Create client, setup session (acquire cookies)
	client := &http.Client{Jar: cookies}
	cfg.SetClient(client)

	baseURL := os.Getenv("BANNER_BASE_URL")
	cfg.SetBaseURL(baseURL)

	apiInstance := api.New(cfg)
	apiInstance.Setup()

	// Create discord session
	session, err := discordgo.New("Bot " + os.Getenv("BOT_TOKEN"))
	if err != nil {
		log.Err(err).Msg("Invalid bot parameters")
	}

	botInstance := bot.New(session, apiInstance, cfg)
	botInstance.RegisterHandlers()

	// Open discord session
	session.AddHandler(func(s *discordgo.Session, r *discordgo.Ready) {
		log.Info().Str("username", r.User.Username).Str("discriminator", r.User.Discriminator).Str("id", r.User.ID).Str("session", s.State.SessionID).Msg("Bot is logged in")
	})
	err = session.Open()
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Cannot open the session")
	}

	// Setup command handlers
	// Register commands with discord
	arr := zerolog.Arr()
	lo.ForEach(bot.CommandDefinitions, func(cmd *discordgo.ApplicationCommand, _ int) {
		arr.Str(cmd.Name)
	})
	log.Info().Array("commands", arr).Msg("Registering commands")

	// In development, use test server, otherwise empty (global) for command registration
	guildTarget := ""
	if cfg.IsDevelopment {
		guildTarget = os.Getenv("BOT_TARGET_GUILD")
	}

	// Register commands
	existingCommands, err := session.ApplicationCommands(session.State.User.ID, guildTarget)
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Cannot get existing commands")
	}
	newCommands, err := session.ApplicationCommandBulkOverwrite(session.State.User.ID, guildTarget, bot.CommandDefinitions)
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Cannot register commands")
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
	err = apiInstance.TryReloadTerms()
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Cannot fetch terms on startup")
	}

	// Launch a goroutine to scrape the banner system periodically
	go func() {
		for {
			err := apiInstance.Scrape()
			if err != nil {
				log.Err(err).Stack().Msg("Periodic Scrape Failed")
			}

			time.Sleep(3 * time.Minute)
		}
	}()

	// Close session, ensure http client closes idle connections
	defer session.Close()
	defer client.CloseIdleConnections()

	// Setup signal handler channel
	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt)    // Ctrl+C signal
	signal.Notify(stop, syscall.SIGTERM) // Container stop signal

	// Wait for signal (indefinite)
	closingSignal := <-stop
	botInstance.SetClosing() // TODO: Switch to atomic lock with forced close after 10 seconds

	// Defers are called after this
	log.Warn().Str("signal", closingSignal.String()).Msg("Gracefully shutting down")
}
