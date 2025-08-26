// Package bot provides the core functionality for the Discord bot.
package bot

import (
	"banner/internal/api"
	"banner/internal/config"
	"fmt"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/rs/zerolog/log"
)

// Bot represents the state of the Discord bot.
type Bot struct {
	Session   *discordgo.Session
	API       *api.API
	Config    *config.Config
	isClosing bool
}

// New creates a new Bot instance.
func New(s *discordgo.Session, a *api.API, c *config.Config) *Bot {
	return &Bot{Session: s, API: a, Config: c}
}

// SetClosing marks the bot as closing, preventing new commands from being processed.
func (b *Bot) SetClosing() {
	b.isClosing = true
}

// GetSession ensures a valid session is available and selects the default term.
func (b *Bot) GetSession() (string, error) {
	sessionID := b.API.EnsureSession()
	term := b.API.DefaultTerm(time.Now()).ToString()

	log.Info().Str("term", term).Str("sessionID", sessionID).Msg("Setting selected term")
	err := b.API.SelectTerm(term, sessionID)
	if err != nil {
		return "", fmt.Errorf("failed to select term while generating session ID: %w", err)
	}

	return sessionID, nil
}
