package main

import (
	"flag"
	"net/http"
	"net/http/cookiejar"
	"os"
	"os/signal"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/joho/godotenv"
	log "github.com/sirupsen/logrus"
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

type MeetingTimeFaculty struct {
	bannerId    int
	category    string
	displayName string
	email       string
	primary     bool
}

type MeetingTimeResponse struct {
	faculty                []MeetingTimeFaculty
	weekdays               map[time.Weekday]bool
	campus                 string
	campusDescription      string
	creditHours            int
	building               string
	buildingDescription    string
	room                   string
	timeStart              NaiveTime
	timeEnd                NaiveTime
	dateStart              time.Time
	dateEnd                time.Time
	hoursPerWeek           float32
	meetingScheduleType    string
	meetingType            string
	meetingTypeDescription string
}

func main() {
	// Load environment variables
	if err := godotenv.Load(); err != nil {
		log.WithFields(log.Fields{
			"error": err,
		}).Warn("Error loading .env file")
	}
	baseURL = os.Getenv("BANNER_BASE_URL")

	cookies, err := cookiejar.New(nil)
	if err != nil {
		log.WithField("error", err).Fatal(err)
	}

	client = http.Client{Jar: cookies}
	setup(cookies)

	session, err = discordgo.New("Bot " + os.Getenv("BOT_TOKEN"))
	if err != nil {
		log.WithField("error", err).Fatal("Invalid bot parameters")
	}

	session.AddHandler(func(s *discordgo.Session, r *discordgo.Ready) {
		log.WithFields(log.Fields{
			"username":      r.User.Username,
			"discriminator": r.User.Discriminator,
			"id":            r.User.ID,
			"session":       s.State.SessionID,
		}).Info("Bot is logged in")
	})
	err = session.Open()
	if err != nil {
		log.Fatalf("Cannot open the session: %v", err)
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
			log.Panicf("Cannot create '%v' command: %v", v.Name, err)
		}
		registeredCommands[i] = cmd
	}

	defer session.Close()

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt)
	log.Println("Press Ctrl+C to exit")
	<-stop

	if *RemoveCommands {
		log.Printf("Removing %d command%s...\n", len(registeredCommands), Plural(len(registeredCommands)))

		for _, v := range registeredCommands {
			err := session.ApplicationCommandDelete(session.State.User.ID, os.Getenv("BOT_TARGET_GUILD"), v.ID)
			if err != nil {
				log.Panicf("Cannot delete '%v' command: %v", v.Name, err)
			}
		}
	}

	log.Println("Gracefully shutting down.")

}
