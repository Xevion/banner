package main

import (
	"fmt"
	"time"

	"github.com/bwmarrin/discordgo"
)

var (
	commandDefinitions = []*discordgo.ApplicationCommand{TermCommandDefinition, TimeCommandDefinition}
	commandHandlers    = map[string]func(s *discordgo.Session, i *discordgo.InteractionCreate){
		"time":   TimeCommandHandler,
		"term":   TermCommandHandler,
		"search": SearchCommandHandler,
	}
	minLength = 0
)

var SearchCommandDefinition = &discordgo.ApplicationCommand{
	Name:        "search",
	Description: "Search for a course",
	Options: []*discordgo.ApplicationCommandOption{
		{
			Type:        discordgo.ApplicationCommandOptionString,
			MinLength:   &minLength,
			MaxLength:   8,
			Name:        "query",
			Description: "Search query",
			Required:    true,
		},
	},
}

func SearchCommandHandler(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
	query := Query{
		keywords: &[]string{"Computer Science"},
	}
	courses := Search(query, 0, 5, "", false)
	fetch_time := time.Now()
	fields := []*discordgo.MessageEmbedField{}

	for _, course := range courses.Data {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "Name",
			Value:  course.CourseTitle,
			Inline: true,
		}, &discordgo.MessageEmbedField{
			Name:   "CRN",
			Value:  "12345",
			Inline: true,
		}, &discordgo.MessageEmbedField{
			Name:   "Credits",
			Value:  "3",
			Inline: true,
		})
	}

	session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Footer: &discordgo.MessageEmbedFooter{
						Text: fmt.Sprintf("Fetched at %s", fetch_time.Format("Monday, January 2, 2006 at 3:04:05PM")),
					},
					Description: "",
					Fields:      fields,
				},
			},
			AllowedMentions: &discordgo.MessageAllowedMentions{},
		},
	})
}

var TermCommandDefinition = &discordgo.ApplicationCommand{
	Name:        "term",
	Description: "Guess the current term, or search for a specific term",
	Options: []*discordgo.ApplicationCommandOption{
		{
			Type:        discordgo.ApplicationCommandOptionString,
			MinLength:   &minLength,
			MaxLength:   8,
			Name:        "term",
			Description: "Term to search for",
			Required:    true,
		},
	},
}

func TermCommandHandler(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
	GetTerms("", 1, 25)

	session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Content: fmt.Sprintf("```json\n%s```", "{\n  \"name\": \"Term\",\n  \"value\": \"202420\"\n}"),
		},
	})
}

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
	},
}

func TimeCommandHandler(s *discordgo.Session, i *discordgo.InteractionCreate) {
	fetch_time := time.Now()
	crn := i.ApplicationCommandData().Options[0].IntValue()
	courseMeetingTime, err := GetCourseMeetingTime(202420, int(crn))
	if err != nil {
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Error getting meeting time",
			},
		})
		return
	}

	duration := courseMeetingTime.timeEnd.Sub(courseMeetingTime.timeStart)

	s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Footer: &discordgo.MessageEmbedFooter{
						Text: fmt.Sprintf("Fetched at %s", fetch_time.Format("Monday, January 2, 2006 at 3:04:05PM")),
					},
					Description: "",
					Fields: []*discordgo.MessageEmbedField{
						{
							Name:  "Start Date",
							Value: courseMeetingTime.dateStart.Format("Monday, January 2, 2006"),
						},
						{
							Name:  "End Date",
							Value: courseMeetingTime.dateEnd.Format("Monday, January 2, 2006"),
						},
						{
							Name:  "Start/End Time",
							Value: fmt.Sprintf("%s - %s (%d min)", courseMeetingTime.timeStart.String(), courseMeetingTime.timeEnd.String(), int64(duration.Minutes())),
						},
						{
							Name:  "Days of Week",
							Value: WeekdaysToString(courseMeetingTime.weekdays),
						},
					},
				},
			},
			AllowedMentions: &discordgo.MessageAllowedMentions{},
		},
	})
}
