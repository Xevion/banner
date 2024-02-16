package main

import (
	"fmt"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/pkg/errors"
	"github.com/rs/zerolog/log"
	"github.com/samber/lo"
)

var (
	commandDefinitions = []*discordgo.ApplicationCommand{TermCommandDefinition, TimeCommandDefinition, SearchCommandDefinition, IcsCommandDefinition}
	commandHandlers    = map[string]func(s *discordgo.Session, i *discordgo.InteractionCreate) error{
		TimeCommandDefinition.Name:   TimeCommandHandler,
		TermCommandDefinition.Name:   TermCommandHandler,
		SearchCommandDefinition.Name: SearchCommandHandler,
		IcsCommandDefinition.Name:    IcsCommandHandler,
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
			Type:        discordgo.ApplicationCommandOptionString,
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
			var low, high int
			var err error
			valueRaw := option.StringValue()

			// 4 digit code
			if len(valueRaw) == 4 {
				low, err = strconv.Atoi(valueRaw)
				if err != nil {
					return errors.Wrap(err, "error parsing course code")
				}

				high = low
			}

			// Partially/fully specified range
			if strings.Contains(valueRaw, "-") {
				match := regexp.MustCompile(`(\d{4})-(\d{4})?`).FindSubmatch([]byte(valueRaw))

				// If not 2 or 3 matches, it's invalid
				if len(match) != 3 && len(match) != 4 {
					return fmt.Errorf("invalid range format: %s", match[0])
				}

				low, err = strconv.Atoi(string(match[1]))
				if err != nil {
					return errors.Wrap(err, "error parsing course code (low)")
				}

				// If there's not a high value, set it to the low value
				if len(match) == 2 {
					high = low
				} else {
					high, err = strconv.Atoi(string(match[2]))
					if err != nil {
						return errors.Wrap(err, "error parsing course code (high)")
					}
				}
			}

			// TODO: #xxx, ##xx, ###x format
			query.CourseNumbers(low, high)
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
		displayName := course.Faculty[0].DisplayName
		categoryLink := fmt.Sprintf("[%s](https://catalog.utsa.edu/undergraduate/coursedescriptions/%s/)", course.Subject, strings.ToLower(course.Subject))
		classLink := fmt.Sprintf("[%s-%s](https://catalog.utsa.edu/search/?P=%s%%20%s)", course.CourseNumber, course.SequenceNumber, course.Subject, course.CourseNumber)
		professorLink := fmt.Sprintf("[%s](https://google.com)", displayName)

		identifierText := fmt.Sprintf("%s %s (CRN %s)\n%s", categoryLink, classLink, course.CourseReferenceNumber, professorLink)
		meetings := course.MeetingsFaculty[0]

		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "Identifier",
			Value:  identifierText,
			Inline: true,
		}, &discordgo.MessageEmbedField{
			Name:   "Name",
			Value:  course.CourseTitle,
			Inline: true,
		}, &discordgo.MessageEmbedField{
			Name:   "Meeting Time",
			Value:  meetings.String(),
			Inline: true,
		},
		)
	}

	err = session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Footer:      GetFooter(fetch_time),
					Description: fmt.Sprintf("%d Classes", courses.TotalCount),
					Fields:      fields[:min(25, len(fields))],
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

	fetch_time := time.Now()

	err = session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Footer:      GetFooter(fetch_time),
					Description: fmt.Sprintf("%d Terms", len(terms)),
					Fields:      fields[:min(25, len(fields))],
				},
			},
			AllowedMentions: &discordgo.MessageAllowedMentions{},
		},
	})

	return err
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

	meetingTimes, err := GetCourseMeetingTime(202420, int(crn))
	if err != nil {
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Error getting meeting time",
			},
		})
		return err
	}

	meetingTime := meetingTimes[0]
	duration := meetingTime.EndTime().Sub(meetingTime.StartTime())

	s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Footer:      GetFooter(fetch_time),
					Description: "",
					Fields: []*discordgo.MessageEmbedField{
						{
							Name:  "Start Date",
							Value: meetingTime.StartDay().Format("Monday, January 2, 2006"),
						},
						{
							Name:  "End Date",
							Value: meetingTime.EndDay().Format("Monday, January 2, 2006"),
						},
						{
							Name:  "Start/End Time",
							Value: fmt.Sprintf("%s - %s (%d min)", meetingTime.StartTime().String(), meetingTime.EndTime().String(), int64(duration.Minutes())),
						},
						{
							Name:  "Days of Week",
							Value: WeekdaysToString(meetingTime.Days()),
						},
					},
				},
			},
			AllowedMentions: &discordgo.MessageAllowedMentions{},
		},
	})
	return nil
}

var IcsCommandDefinition = &discordgo.ApplicationCommand{
	Name:        "ics",
	Description: "Generate an ICS file for a course",
	Options: []*discordgo.ApplicationCommandOption{
		{
			Type:        discordgo.ApplicationCommandOptionInteger,
			Name:        "crn",
			Description: "Course Reference Number",
			Required:    true,
		},
	},
}

func IcsCommandHandler(s *discordgo.Session, i *discordgo.InteractionCreate) error {
	crn := i.ApplicationCommandData().Options[0].IntValue()

	course, err := GetCourse(strconv.Itoa(int(crn)))
	if err != nil {
		return fmt.Errorf("Error retrieving course data: %w", err)
	}

	meetingTimes, err := GetCourseMeetingTime(202420, int(crn))
	if err != nil {
		return fmt.Errorf("Error requesting meeting time: %w", err)
	}

	if len(meetingTimes) == 0 {
		return fmt.Errorf("unexpected - no meeting time data found for course")
	}

	// Check if the course has any meeting times
	_, exists := lo.Find(meetingTimes, func(mt MeetingTimeResponse) bool {
		switch mt.MeetingTime.MeetingType {
		case "ID", "OA":
			return false
		default:
			return true
		}
	})

	if !exists {
		log.Warn().Str("crn", course.CourseReferenceNumber).Msg("Non-meeting course requested for ICS file")
		RespondError(s, i.Interaction, "The course requested does not meet at a defined moment in time.", nil)
		return nil
	}

	events := []string{}
	for _, meeting := range meetingTimes {
		now := time.Now().In(CentralTimeLocation)
		uid := fmt.Sprintf("%d-%s@ical.banner.xevion.dev", now.Unix(), meeting.CourseReferenceNumber)

		startDay := meeting.StartDay()
		startTime := meeting.StartTime()
		endTime := meeting.EndTime()
		dtStart := time.Date(startDay.Year(), startDay.Month(), startDay.Day(), int(startTime.Hours), int(startTime.Minutes), 0, 0, CentralTimeLocation)
		dtEnd := time.Date(startDay.Year(), startDay.Month(), startDay.Day(), int(endTime.Hours), int(endTime.Minutes), 0, 0, CentralTimeLocation)

		endDay := meeting.EndDay()
		until := time.Date(endDay.Year(), endDay.Month(), endDay.Day(), 23, 59, 59, 0, CentralTimeLocation)

		summary := fmt.Sprintf("%s %s %s", course.Subject, course.CourseNumber, course.CourseTitle)
		description := fmt.Sprintf("Instructor: %s\nSection: %s\nCRN: %s", course.Faculty[0].DisplayName, course.SequenceNumber, meeting.CourseReferenceNumber)
		location := meeting.PlaceString()

		event := fmt.Sprintf(`BEGIN:VEVENT
DTSTAMP:%s
UID:%s
DTSTART;TZID=America/Chicago:%s
RRULE:FREQ=WEEKLY;BYDAY=%s;UNTIL=%s
DTEND;TZID=America/Chicago:%s
SUMMARY:%s
DESCRIPTION:%s
LOCATION:%s
END:VEVENT`, now.Format(ICalTimestampFormatLocal), uid, dtStart.Format(ICalTimestampFormatLocal), meeting.ByDay(), until.Format(ICalTimestampFormatLocal), dtEnd.Format(ICalTimestampFormatLocal), summary, strings.Replace(description, "\n", `\n`, -1), location)

		events = append(events, event)
	}

	// TODO: Make this dynamically requested, parsed & cached from tzurl.org
	vTimezone := `BEGIN:VTIMEZONE
TZID:America/Chicago
LAST-MODIFIED:20231222T233358Z
TZURL:https://www.tzurl.org/zoneinfo-outlook/America/Chicago
X-LIC-LOCATION:America/Chicago
BEGIN:DAYLIGHT
TZNAME:CDT
TZOFFSETFROM:-0600
TZOFFSETTO:-0500
DTSTART:19700308T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU
END:DAYLIGHT
BEGIN:STANDARD
TZNAME:CST
TZOFFSETFROM:-0500
TZOFFSETTO:-0600
DTSTART:19701101T020000
RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU
END:STANDARD
END:VTIMEZONE`

	ics := fmt.Sprintf(`BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//xevion//Banner Discord Bot//EN
CALSCALE:GREGORIAN
%s
%s
END:VCALENDAR`, vTimezone, strings.Join(events, "\n"))

	session.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Files: []*discordgo.File{
				{
					Name:        fmt.Sprintf("%d.ics", crn),
					ContentType: "text/calendar",
					Reader:      strings.NewReader(ics),
				},
			},
			AllowedMentions: &discordgo.MessageAllowedMentions{},
		},
	})
	return nil
}
