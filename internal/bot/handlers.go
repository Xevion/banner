package bot

import (
	"banner/internal"
	"fmt"

	"github.com/bwmarrin/discordgo"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func (b *Bot) RegisterHandlers() {
	b.Session.AddHandler(func(internalSession *discordgo.Session, interaction *discordgo.InteractionCreate) {
		// Handle commands during restart (highly unlikely, but just in case)
		if b.isClosing {
			err := internal.RespondError(internalSession, interaction.Interaction, "Bot is currently restarting, try again later.", nil)
			if err != nil {
				log.Error().Err(err).Msg("Failed to respond with restart error feedback")
			}
			return
		}

		name := interaction.ApplicationCommandData().Name
		if handler, ok := CommandHandlers[name]; ok {
			// Build dict of options for the log
			options := zerolog.Dict()
			for _, option := range interaction.ApplicationCommandData().Options {
				options.Str(option.Name, fmt.Sprintf("%v", option.Value))
			}

			event := log.Info().Str("name", name).Str("user", internal.GetUser(interaction).Username).Dict("options", options)

			// If the command was invoked in a guild, add guild & channel info to the log
			if interaction.Member != nil {
				guild := zerolog.Dict()
				guild.Str("id", interaction.GuildID)
				guild.Str("name", internal.GetGuildName(b.Config, internalSession, interaction.GuildID))
				event.Dict("guild", guild)

				channel := zerolog.Dict()
				channel.Str("id", interaction.ChannelID)
				guild.Str("name", internal.GetChannelName(b.Config, internalSession, interaction.ChannelID))
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

			// Prepare to recover
			defer func() {
				if err := recover(); err != nil {
					log.Error().Stack().Str("commandName", name).Interface("detail", err).Msg("Command Handler Panic")

					// Respond with error
					err := internal.RespondError(internalSession, interaction.Interaction, "Unexpected Error: command handler panic", nil)
					if err != nil {
						log.Error().Stack().Str("commandName", name).Err(err).Msg("Failed to respond with panic error feedback")
					}
				}
			}()

			// Call handler
			err := handler(b, internalSession, interaction)

			// Log & respond error
			if err != nil {
				// TODO: Find a way to merge the response with the handler's error
				log.Error().Str("commandName", name).Err(err).Msg("Command Handler Error")

				// Respond with error
				err = internal.RespondError(internalSession, interaction.Interaction, fmt.Sprintf("Unexpected Error: %s", err.Error()), nil)
				if err != nil {
					log.Error().Stack().Str("commandName", name).Err(err).Msg("Failed to respond with error feedback")
				}
			}

		} else {
			log.Error().Stack().Str("commandName", name).Msg("Command Interaction Has No Handler")

			// Respond with error
			internal.RespondError(internalSession, interaction.Interaction, "Unexpected Error: interaction has no handler", nil)
		}
	})
}
