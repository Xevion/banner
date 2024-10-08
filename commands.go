package main

import (
	"fmt"
	"net/url"
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
			Type:         discordgo.ApplicationCommandOptionString,
			MinLength:    GetIntPointer(0),
			MaxLength:    48,
			Name:         "title",
			Description:  "Course Title (exact, use autocomplete)",
			Required:     false,
			Autocomplete: true,
		},
		{
			Type:        discordgo.ApplicationCommandOptionString,
			Name:        "code",
			MinLength:   GetIntPointer(4),
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
		},
		{
			Type:         discordgo.ApplicationCommandOptionString,
			Name:         "instructor",
			Description:  "Instructor Name",
			Required:     false,
			Autocomplete: true,
		},
		{
			Type:         discordgo.ApplicationCommandOptionString,
			Name:         "subject",
			Description:  "Subject (e.g. Computer Science/CS, Mathematics/MAT)",
			Required:     false,
			Autocomplete: true,
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
			var (
				low  = -1
				high = -1
			)
			var err error
			valueRaw := strings.TrimSpace(option.StringValue())

			// Partially/fully specified range
			if strings.Contains(valueRaw, "-") {
				match := regexp.MustCompile(`(\d{1,4})-(\d{1,4})?`).FindSubmatch([]byte(valueRaw))

				if match == nil {
					return fmt.Errorf("invalid range format: %s", valueRaw)
				}

				// If not 2 or 3 matches, it's invalid
				if len(match) != 3 && len(match) != 4 {
					return fmt.Errorf("invalid range format: %s", match[0])
				}

				low, err = strconv.Atoi(string(match[1]))
				if err != nil {
					return errors.Wrap(err, "error parsing course code (low)")
				}

				// If there's not a high value, set it to max (open ended)
				if len(match) == 2 || len(match[2]) == 0 {
					high = 9999
				} else {
					high, err = strconv.Atoi(string(match[2]))
					if err != nil {
						return errors.Wrap(err, "error parsing course code (high)")
					}
				}
			}

			// #xxx, ##xx, ###x format (34xx -> 3400-3499)
			if strings.Contains(valueRaw, "x") {
				if len(valueRaw) != 4 {
					return fmt.Errorf("code range format invalid: must be 1 or more digits followed by x's (%s)", valueRaw)
				}

				match := regexp.MustCompile(`\d{1,}([xX]{1,3})`).Match([]byte(valueRaw))
				if !match {
					return fmt.Errorf("code range format invalid: must be 1 or more digits followed by x's (%s)", valueRaw)
				}

				// Replace x's with 0's
				low, err = strconv.Atoi(strings.Replace(valueRaw, "x", "0", -1))
				if err != nil {
					return errors.Wrap(err, "error parsing implied course code (low)")
				}

				// Replace x's with 9's
				high, err = strconv.Atoi(strings.Replace(valueRaw, "x", "9", -1))
				if err != nil {
					return errors.Wrap(err, "error parsing implied course code (high)")
				}
			} else if len(valueRaw) == 4 {
				// 4 digit code
				low, err = strconv.Atoi(valueRaw)
				if err != nil {
					return errors.Wrap(err, "error parsing course code")
				}

				high = low
			}

			if low == -1 || high == -1 {
				return fmt.Errorf("course code range invalid (%s)", valueRaw)
			}

			if low > high {
				return fmt.Errorf("course code range is invalid: low is greater than high (%d > %d)", low, high)
			}

			if low < 1000 || high < 1000 || low > 9999 || high > 9999 {
				return fmt.Errorf("course code range is invalid: must be 1000-9999 (%d-%d)", low, high)
			}

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
		professorLink := fmt.Sprintf("[%s](https://www.ratemyprofessors.com/search/professors/1516?q=%s)", displayName, url.QueryEscape(displayName))

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

	// Blue if there are results, orange if there are none
	color := 0x0073FF
	if courses.TotalCount == 0 {
		color = 0xFF6500
	}

	err = session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Footer:      GetFetchedFooter(fetch_time),
					Description: p.Sprintf("%d Class%s", courses.TotalCount, Plurale(courses.TotalCount)),
					Fields:      fields[:min(25, len(fields))],
					Color:       color,
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
			MinLength:   GetIntPointer(0),
			MaxLength:   8,
			Name:        "search",
			Description: "Term to search for",
			Required:    false,
		},
		{
			Type:        discordgo.ApplicationCommandOptionInteger,
			Name:        "page",
			Description: "Page Number",
			Required:    false,
			MinValue:    GetFloatPointer(1),
		},
	},
}

func TermCommandHandler(session *discordgo.Session, interaction *discordgo.InteractionCreate) error {
	data := interaction.ApplicationCommandData()

	searchTerm := ""
	pageNumber := 1

	for _, option := range data.Options {
		switch option.Name {
		case "search":
			searchTerm = option.StringValue()
		case "page":
			pageNumber = int(option.IntValue())
		default:
			log.Warn().Str("option", option.Name).Msg("Unexpected option in term command")
		}
	}

	termResult, err := GetTerms(searchTerm, pageNumber, 25)

	if err != nil {
		RespondError(session, interaction.Interaction, "Error while fetching terms", err)
		return err
	}

	fields := []*discordgo.MessageEmbedField{}

	for _, t := range termResult {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   t.Description,
			Value:  t.Code,
			Inline: true,
		})
	}

	fetch_time := time.Now()

	if len(fields) > 25 {
		log.Warn().Int("count", len(fields)).Msg("Too many fields in term command (trimmed)")
	}

	err = session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Footer:      GetFetchedFooter(fetch_time),
					Description: p.Sprintf("%d of %d term%s (page %d)", len(termResult), len(terms), Plural(len(terms)), pageNumber),
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

	// Fix static term
	meetingTimes, err := GetCourseMeetingTime(202510, int(crn))
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
					Footer:      GetFetchedFooter(fetch_time),
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

	// Fix static term
	meetingTimes, err := GetCourseMeetingTime(202510, int(crn))
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
					Name:        fmt.Sprintf("%s-%s-%s_%s.ics", course.Subject, course.CourseNumber, course.SequenceNumber, course.CourseReferenceNumber),
					ContentType: "text/calendar",
					Reader:      strings.NewReader(ics),
				},
			},
			AllowedMentions: &discordgo.MessageAllowedMentions{},
		},
	})
	return nil
}
