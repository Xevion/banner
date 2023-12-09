package main

import (
	"strconv"

	"github.com/bwmarrin/discordgo"
)

var commandDefinitions = []*discordgo.ApplicationCommand{TimeCommandDefinition}

var TimeCommandDefinition = &discordgo.ApplicationCommand{
	Name:        "time",
	Description: "Get Class Meeting Time",
	Options: []*discordgo.ApplicationCommandOption{
		{
			Type:        discordgo.ApplicationCommandOptionInteger,
			Name:        "crn",
			Description: "Course Reference Number",
			Required:    true,
		},
		{
			Type:        discordgo.ApplicationCommandOptionString,
			Name:        "term",
			Description: "Term",
			Required:    false,
		},
	},
}

func TimeCommandHandler(s *discordgo.Session, i *discordgo.InteractionCreate) {
	crn := i.ApplicationCommandData().Options[0].IntValue()
	_, err := GetCourseMeetingTime(202420, int(crn))

	if err != nil {
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Error getting meeting time",
			},
		})
		return
	}
	s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Title:       "CRN " + strconv.Itoa(int(crn)),
					Description: "",
					Fields:      []*discordgo.MessageEmbedField{},
				},
			},
			AllowedMentions: &discordgo.MessageAllowedMentions{},
		},
	})
}
