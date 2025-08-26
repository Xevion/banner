package bot

import (
	"banner/internal/api"
	"banner/internal/config"
	"fmt"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/rs/zerolog/log"
)

type Bot struct {
	Session   *discordgo.Session
	API       *api.API
	Config    *config.Config
	isClosing bool
}

func New(s *discordgo.Session, a *api.API, c *config.Config) *Bot {
	return &Bot{Session: s, API: a, Config: c}
}

func (b *Bot) SetClosing() {
	b.isClosing = true
}

func (b *Bot) GetSession() (string, error) {
	sessionID := b.API.EnsureSession()
	term := api.Default(time.Now()).ToString()

	log.Info().Str("term", term).Str("sessionID", sessionID).Msg("Setting selected term")
	err := b.API.SelectTerm(term, sessionID)
	if err != nil {
		return "", fmt.Errorf("failed to select term while generating session ID: %w", err)
	}

	return sessionID, nil
}
