package main

import (
	"fmt"
	"strings"
	"time"

	"github.com/bwmarrin/discordgo"
)

var (
	commandDefinitions = []*discordgo.ApplicationCommand{TermCommandDefinition, TimeCommandDefinition, SearchCommandDefinition}
	commandHandlers    = map[string]func(s *discordgo.Session, i *discordgo.InteractionCreate) error{
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
			Name:        "title",
			Description: "Course Title",
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
		{
			Type:        discordgo.ApplicationCommandOptionString,
			Name:        "keywords",
			Description: "Keywords in Title or Description (space separated)",
			Required:    false,
		},
	},
}

func SearchCommandHandler(session *discordgo.Session, interaction *discordgo.InteractionCreate) error {
	data := interaction.ApplicationCommandData()
	query := NewQuery().Credits(3, 6)

	for _, option := range data.Options {
		switch option.Name {
		case "title":
			query.Title(option.StringValue())
		case "code":
			// TODO: Handle & parse course codes properly
		case "keywords":
			query.Keywords(
				strings.Split(option.StringValue(), " "),
			)
		case "max":
			query.MaxResults(
				min(8, int(option.IntValue())),
			)
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
		return err
	}

	fetch_time := time.Now()
	fields := []*discordgo.MessageEmbedField{}

	for _, course := range courses.Data {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "Identifier",
			Value:  fmt.Sprintf("%s %s-%s [%s]", course.Subject, course.CourseNumber, course.SequenceNumber, course.CourseReferenceNumber),
			Inline: true,
		}, &discordgo.MessageEmbedField{
			Name:   "Name",
			Value:  course.CourseTitle,
			Inline: true,
		}, &discordgo.MessageEmbedField{
			Name:   "Meeting Time",
			Value:  "MWF 11AM-12:15PM",
			Inline: true,
		})
	}

	err = session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
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

	return err
}

var TermCommandDefinition = &discordgo.ApplicationCommand{
	Name:        "terms",
	Description: "Guess the current term, or search for a specific term",
	Options: []*discordgo.ApplicationCommandOption{
		{
			Type:        discordgo.ApplicationCommandOptionString,
			MinLength:   GetPointer(0),
			MaxLength:   8,
			Name:        "search",
			Description: "Term to search for",
			Required:    false,
		},
	},
}

func TermCommandHandler(session *discordgo.Session, interaction *discordgo.InteractionCreate) error {
	data := interaction.ApplicationCommandData()

	var searchTerm string
	if len(data.Options) == 1 {
		searchTerm = data.Options[0].StringValue()
	}

	terms, err := GetTerms(searchTerm, 1, 25)

	if err != nil {
		RespondError(session, interaction.Interaction, "Error while fetching terms", err)
		return err
	}

	fields := []*discordgo.MessageEmbedField{}

	for _, t := range terms {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "ID",
			Value:  t.Code,
			Inline: true,
		}, &discordgo.MessageEmbedField{
			Name:   "Description",
			Value:  t.Description,
			Inline: true,
		})
	}

	return nil
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

func TimeCommandHandler(s *discordgo.Session, i *discordgo.InteractionCreate) error {
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
		return err
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
	return nil
}
