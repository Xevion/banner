// Package models provides the data structures for the Banner API.
package models

import (
	"banner/internal"
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
	"time"

	log "github.com/rs/zerolog/log"
)

// FacultyItem represents a faculty member associated with a course.
type FacultyItem struct {
	BannerID              string  `json:"bannerId"`
	Category              *string `json:"category"`
	Class                 string  `json:"class"`
	CourseReferenceNumber string  `json:"courseReferenceNumber"`
	DisplayName           string  `json:"displayName"`
	Email                 string  `json:"emailAddress"`
	Primary               bool    `json:"primaryIndicator"`
	Term                  string  `json:"term"`
}

// MeetingTimeResponse represents the meeting time information for a course.
type MeetingTimeResponse struct {
	Category              *string `json:"category"`
	Class                 string  `json:"class"`
	CourseReferenceNumber string  `json:"courseReferenceNumber"`
	Faculty               []FacultyItem
	MeetingTime           struct {
		Category string `json:"category"`
		// Some sort of metadata used internally by Banner (net.hedtech.banner.student.schedule.SectionSessionDecorator)
		Class string `json:"class"`
		// The start date of the meeting time in MM/DD/YYYY format (e.g. 01/16/2024)
		StartDate string `json:"startDate"`
		// The end date of the meeting time in MM/DD/YYYY format (e.g. 05/10/2024)
		EndDate string `json:"endDate"`
		// The start time of the meeting time in 24-hour format, hours & minutes, digits only (e.g. 1630)
		BeginTime string `json:"beginTime"`
		// The end time of the meeting time in 24-hour format, hours & minutes, digits only (e.g. 1745)
		EndTime string `json:"endTime"`
		// The room number within the building this course takes place at (e.g. 3.01.08, 200A)
		Room string `json:"room"`
		// The internal identifier for the term this course takes place in (e.g. 202420)
		Term string `json:"term"`
		// The internal identifier for the building this course takes place at (e.g. SP1)
		Building string `json:"building"`
		// The long name of the building this course takes place at (e.g. San Pedro I - Data Science)
		BuildingDescription string `json:"buildingDescription"`
		// The internal identifier for the campus this course takes place at (e.g. 1DT)
		Campus string `json:"campus"`
		// The long name of the campus this course takes place at (e.g. Main Campus, Downtown Campus)
		CampusDescription     string `json:"campusDescription"`
		CourseReferenceNumber string `json:"courseReferenceNumber"`
		// The number of credit hours this class is worth (assumably)
		CreditHourSession float64 `json:"creditHourSession"`
		// The number of hours per week this class meets (e.g. 2.5)
		HoursWeek float64 `json:"hoursWeek"`
		// Unknown meaning - e.g. AFF, AIN, AHB, FFF, AFF, EFF, DFF, IFF, EHB, JFF, KFF, BFF, BIN
		MeetingScheduleType string `json:"meetingScheduleType"`
		// The short identifier for the meeting type (e.g. FF, HB, OS, OA)
		MeetingType string `json:"meetingType"`
		// The long name of the meeting type (e.g. Traditional in-person)
		MeetingTypeDescription string `json:"meetingTypeDescription"`
		// A boolean indicating if the class will meet on each Monday of the term
		Monday bool `json:"monday"`
		// A boolean indicating if the class will meet on each Tuesday of the term
		Tuesday bool `json:"tuesday"`
		// A boolean indicating if the class will meet on each Wednesday of the term
		Wednesday bool `json:"wednesday"`
		// A boolean indicating if the class will meet on each Thursday of the term
		Thursday bool `json:"thursday"`
		// A boolean indicating if the class will meet on each Friday of the term
		Friday bool `json:"friday"`
		// A boolean indicating if the class will meet on each Saturday of the term
		Saturday bool `json:"saturday"`
		// A boolean indicating if the class will meet on each Sunday of the term
		Sunday bool `json:"sunday"`
	} `json:"meetingTime"`
	Term string `json:"term"`
}

// String returns a formatted string representation of the meeting time.
func (m *MeetingTimeResponse) String() string {
	switch m.MeetingTime.MeetingType {
	case "HB":
		return fmt.Sprintf("%s\nHybrid %s", m.TimeString(), m.PlaceString())
	case "H2":
		return fmt.Sprintf("%s\nHybrid %s", m.TimeString(), m.PlaceString())
	case "H1":
		return fmt.Sprintf("%s\nHybrid %s", m.TimeString(), m.PlaceString())
	case "OS":
		return fmt.Sprintf("%s\nOnline Only", m.TimeString())
	case "OA":
		return "No Time\nOnline Asynchronous"
	case "OH":
		return fmt.Sprintf("%s\nOnline Partial", m.TimeString())
	case "ID":
		return "To Be Arranged"
	case "FF":
		return fmt.Sprintf("%s\n%s", m.TimeString(), m.PlaceString())
	}

	// TODO: Add error log
	return "Unknown"
}

// TimeString returns a formatted string of the meeting times (e.g., "MWF 1:00PM-2:15PM").
func (m *MeetingTimeResponse) TimeString() string {
	startTime := m.StartTime()
	endTime := m.EndTime()

	if startTime == nil || endTime == nil {
		return "???"
	}

	return fmt.Sprintf("%s %s-%s", internal.WeekdaysToString(m.Days()), m.StartTime().String(), m.EndTime().String())
}

// PlaceString returns a formatted string representing the location of the meeting.
func (m *MeetingTimeResponse) PlaceString() string {
	mt := m.MeetingTime

	// TODO: Add format case for partial online classes
	if mt.Room == "" {
		return "Online"
	}

	return fmt.Sprintf("%s | %s | %s %s", mt.CampusDescription, mt.BuildingDescription, mt.Building, mt.Room)
}

// Days returns a map of weekdays on which the course meets.
func (m *MeetingTimeResponse) Days() map[time.Weekday]bool {
	days := map[time.Weekday]bool{}

	days[time.Monday] = m.MeetingTime.Monday
	days[time.Tuesday] = m.MeetingTime.Tuesday
	days[time.Wednesday] = m.MeetingTime.Wednesday
	days[time.Thursday] = m.MeetingTime.Thursday
	days[time.Friday] = m.MeetingTime.Friday
	days[time.Saturday] = m.MeetingTime.Saturday

	return days
}

// ByDay returns a comma-separated string of two-letter day abbreviations for the iCalendar RRule.
func (m *MeetingTimeResponse) ByDay() string {
	days := []string{}

	if m.MeetingTime.Sunday {
		days = append(days, "SU")
	}
	if m.MeetingTime.Monday {
		days = append(days, "MO")
	}
	if m.MeetingTime.Tuesday {
		days = append(days, "TU")
	}
	if m.MeetingTime.Wednesday {
		days = append(days, "WE")
	}
	if m.MeetingTime.Thursday {
		days = append(days, "TH")
	}
	if m.MeetingTime.Friday {
		days = append(days, "FR")
	}
	if m.MeetingTime.Saturday {
		days = append(days, "SA")
	}

	return strings.Join(days, ",")
}

const layout = "01/02/2006"

// StartDay returns the start date of the meeting as a time.Time object.
// This method is not cached and will panic if the date cannot be parsed.
func (m *MeetingTimeResponse) StartDay() time.Time {
	t, err := time.Parse(layout, m.MeetingTime.StartDate)
	if err != nil {
		log.Panic().Stack().Err(err).Str("raw", m.MeetingTime.StartDate).Msg("Cannot parse start date")
	}
	return t
}

// EndDay returns the end date of the meeting as a time.Time object.
// This method is not cached and will panic if the date cannot be parsed.
func (m *MeetingTimeResponse) EndDay() time.Time {
	t, err := time.Parse(layout, m.MeetingTime.EndDate)
	if err != nil {
		log.Panic().Stack().Err(err).Str("raw", m.MeetingTime.EndDate).Msg("Cannot parse end date")
	}
	return t
}

// StartTime returns the start time of the meeting as a NaiveTime object.
// This method is not cached and will panic if the time cannot be parsed.
func (m *MeetingTimeResponse) StartTime() *internal.NaiveTime {
	raw := m.MeetingTime.BeginTime
	if raw == "" {
		log.Panic().Stack().Msg("Start time is empty")
	}

	value, err := strconv.ParseUint(raw, 10, 32)
	if err != nil {
		log.Panic().Stack().Err(err).Str("raw", raw).Msg("Cannot parse start time integer")
	}

	return internal.ParseNaiveTime(value)
}

// EndTime returns the end time of the meeting as a NaiveTime object.
// This method is not cached and will panic if the time cannot be parsed.
func (m *MeetingTimeResponse) EndTime() *internal.NaiveTime {
	raw := m.MeetingTime.EndTime
	if raw == "" {
		return nil
	}

	value, err := strconv.ParseUint(raw, 10, 32)
	if err != nil {
		log.Panic().Stack().Err(err).Str("raw", raw).Msg("Cannot parse end time integer")
	}

	return internal.ParseNaiveTime(value)
}

// RRule represents a recurrence rule for an iCalendar event.
type RRule struct {
	Until string
	ByDay string
}

// RRule converts the meeting time to a struct that satisfies the iCalendar RRule format.
func (m *MeetingTimeResponse) RRule() RRule {
	return RRule{
		Until: m.EndDay().UTC().Format("20060102T150405Z"),
		ByDay: m.ByDay(),
	}
}

// SearchResult represents the result of a course search.
type SearchResult struct {
	Success             bool   `json:"success"`
	TotalCount          int    `json:"totalCount"`
	PageOffset          int    `json:"pageOffset"`
	PageMaxSize         int    `json:"pageMaxSize"`
	PathMode            string `json:"pathMode"`
	SearchResultsConfig []struct {
		Config  string `json:"config"`
		Display string `json:"display"`
	} `json:"searchResultsConfig"`
	Data []Course `json:"data"`
}

// Course represents a single course returned from a search.
type Course struct {
	// ID is an internal identifier not used outside of the Banner system.
	ID int `json:"id"`
	// Term is the internal identifier for the term this class is in (e.g. 202420).
	Term string `json:"term"`
	// TermDesc is the human-readable name of the term this class is in (e.g. Fall 2021).
	TermDesc string `json:"termDesc"`
	// CourseReferenceNumber is the unique identifier for a course within a term.
	CourseReferenceNumber string `json:"courseReferenceNumber"`
	// PartOfTerm specifies which part of the term the course is in (e.g. B6, B5).
	PartOfTerm string `json:"partOfTerm"`
	// CourseNumber is the 4-digit code for the course (e.g. 3743).
	CourseNumber string `json:"courseNumber"`
	// Subject is the subject acronym (e.g. CS, AEPI).
	Subject string `json:"subject"`
	// SubjectDescription is the full name of the course subject.
	SubjectDescription string `json:"subjectDescription"`
	// SequenceNumber is the course section (e.g. 001, 002).
	SequenceNumber    string `json:"sequenceNumber"`
	CampusDescription string `json:"campusDescription"`
	// ScheduleTypeDescription is the type of schedule for the course (e.g. Lecture, Seminar).
	ScheduleTypeDescription string `json:"scheduleTypeDescription"`
	CourseTitle             string `json:"courseTitle"`
	CreditHours             int    `json:"creditHours"`
	// MaximumEnrollment is the maximum number of students that can enroll.
	MaximumEnrollment   int     `json:"maximumEnrollment"`
	Enrollment          int     `json:"enrollment"`
	SeatsAvailable      int     `json:"seatsAvailable"`
	WaitCapacity        int     `json:"waitCapacity"`
	WaitCount           int     `json:"waitCount"`
	CrossList           *string `json:"crossList"`
	CrossListCapacity   *int    `json:"crossListCapacity"`
	CrossListCount      *int    `json:"crossListCount"`
	CrossListAvailable  *int    `json:"crossListAvailable"`
	CreditHourHigh      *int    `json:"creditHourHigh"`
	CreditHourLow       *int    `json:"creditHourLow"`
	CreditHourIndicator *string `json:"creditHourIndicator"`
	OpenSection         bool    `json:"openSection"`
	LinkIdentifier      *string `json:"linkIdentifier"`
	IsSectionLinked     bool    `json:"isSectionLinked"`
	// SubjectCourse is the combination of the subject and course number (e.g. CS3443).
	SubjectCourse                  string  `json:"subjectCourse"`
	ReservedSeatSummary            *string `json:"reservedSeatSummary"`
	InstructionalMethod            string  `json:"instructionalMethod"`
	InstructionalMethodDescription string  `json:"instructionalMethodDescription"`
	SectionAttributes              []struct {
		// Class is an internal API class identifier used by Banner.
		Class                 string `json:"class"`
		CourseReferenceNumber string `json:"courseReferenceNumber"`
		// Code for the attribute (e.g., UPPR, ZIEP, AIS).
		Code           string `json:"code"`
		Description    string `json:"description"`
		TermCode       string `json:"termCode"`
		IsZtcAttribute bool   `json:"isZTCAttribute"`
	} `json:"sectionAttributes"`
	Faculty         []FacultyItem         `json:"faculty"`
	MeetingsFaculty []MeetingTimeResponse `json:"meetingsFaculty"`
}

// MarshalBinary implements the encoding.BinaryMarshaler interface.
func (course Course) MarshalBinary() ([]byte, error) {
	return json.Marshal(course)
}
