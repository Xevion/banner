package main

import (
	"fmt"
	"strconv"
	"time"

	"github.com/bwmarrin/discordgo"
)

var (
	commandDefinitions = []*discordgo.ApplicationCommand{TermCommandDefinition, TimeCommandDefinition, SearchCommandDefinition}
	commandHandlers    = map[string]func(s *discordgo.Session, i *discordgo.InteractionCreate){
		TimeCommandDefinition.Name:   TimeCommandHandler,
		TermCommandDefinition.Name:   TermCommandHandler,
		SearchCommandDefinition.Name: SearchCommandHandler,
	}
)

var SearchCommandDefinition = &discordgo.ApplicationCommand{
	Name:        "search",
	Description: "Search for a course",
	Options: []*discordgo.ApplicationCommandOption{
		{
			Type:        discordgo.ApplicationCommandOptionString,
			MinLength:   GetPointer(0),
			MaxLength:   16,
			Name:        "name",
			Description: "Course Name",
			Required:    false,
		},
		{
			Type:        discordgo.ApplicationCommandOptionInteger,
			Name:        "code",
			MinLength:   GetPointer(2),
			Description: "Course Code (e.g. 3743, 3000-3999, 3xxx, 3000-)",
			Required:    false,
		},
		{
			Type:        discordgo.ApplicationCommandOptionInteger,
			Name:        "max",
			Description: "Maximum number of results",
			Required:    false,
		},
	},
}

func SearchCommandHandler(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
	data := interaction.ApplicationCommandData()
	query := NewQuery().Credits(3, 6)

	for _, option := range data.Options {
		switch option.Name {
		case "name":
		case "code":
		case "max":
			query.MaxResults(int(option.IntValue()))
		}
	}

	courses, err := Search(query, "", false)
	if err != nil {
		session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Error searching for courses",
			},
		})
		return
	}

	fetch_time := time.Now()
	fields := []*discordgo.MessageEmbedField{}

	for _, course := range courses.Data {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "Name",
			Value:  course.CourseTitle,
			Inline: true,
		}, &discordgo.MessageEmbedField{
			Name:   "CRN",
			Value:  course.CourseReferenceNumber,
			Inline: true,
		}, &discordgo.MessageEmbedField{
			Name:   "Credits",
			Value:  strconv.Itoa(course.CreditHours),
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
			MinLength:   GetPointer(0),
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
