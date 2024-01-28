package main

import (
	"fmt"
	"strconv"
	"strings"
	"time"

	log "github.com/rs/zerolog/log"
)

const JsonContentType = "application/json"

type FacultyItem struct {
	BannerId              string  `json:"bannerId"`
	Category              *string `json:"category"`
	Class                 string  `json:"class"`
	CourseReferenceNumber string  `json:"courseReferenceNumber"`
	DisplayName           string  `json:"displayName"`
	Email                 string  `json:"emailAddress"`
	Primary               bool    `json:"primaryIndicator"`
	Term                  string  `json:"term"`
}

type MeetingTimeResponse struct {
	Category              *string `json:"category"`
	Class                 string  `json:"class"`
	CourseReferenceNumber string  `json:"courseReferenceNumber"`
	Faculty               []FacultyItem
	MeetingTime           struct {
		Category               string  `json:"category"`
		Class                  string  `json:"class"`
		StartDate              string  `json:"startDate"`
		EndDate                string  `json:"endDate"`
		BeginTime              string  `json:"beginTime"`
		EndTime                string  `json:"endTime"`
		Room                   string  `json:"room"`
		Term                   string  `json:"term"`
		Building               string  `json:"building"`
		BuildingDescription    string  `json:"buildingDescription"`
		Campus                 string  `json:"campus"`
		CampusDescription      string  `json:"campusDescription"`
		CourseReferenceNumber  string  `json:"courseReferenceNumber"`
		CreditHourSession      float64 `json:"creditHourSession"`
		HoursWeek              float64 `json:"hoursWeek"`
		MeetingScheduleType    string  `json:"meetingScheduleType"`
		MeetingType            string  `json:"meetingType"`
		MeetingTypeDescription string  `json:"meetingTypeDescription"`
		Monday                 bool    `json:"monday"`
		Tuesday                bool    `json:"tuesday"`
		Wednesday              bool    `json:"wednesday"`
		Thursday               bool    `json:"thursday"`
		Friday                 bool    `json:"friday"`
		Saturday               bool    `json:"saturday"`
		Sunday                 bool    `json:"sunday"`
	} `json:"meetingTime"`
	Term string `json:"term"`
}

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
		return "TBA"
	case "FF":
		return fmt.Sprintf("%s\n%s", m.TimeString(), m.PlaceString())
	}

	// TODO: Add error log
	return "Unknown"
}

func (m *MeetingTimeResponse) TimeString() string {
	startTime := m.StartTime()
	endTime := m.EndTime()

	if startTime == nil || endTime == nil {
		return "???"
	}

	return fmt.Sprintf("%s %s-%s", WeekdaysToString(m.Days()), m.StartTime().String(), m.EndTime().String())
}

// PlaceString returns a formatted string best representing the place of the meeting time
func (m *MeetingTimeResponse) PlaceString() string {
	if m.MeetingTime.Room == "" {
		return "???"
	}

	return fmt.Sprintf("%s %s", m.MeetingTime.Building, m.MeetingTime.Room)
}

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

// Returns the BYDAY value for the iCalendar RRule format
func (m *MeetingTimeResponse) ByDay() string {
	sb := strings.Builder{}

	if m.MeetingTime.Sunday {
		sb.WriteString("SU,")
	}
	if m.MeetingTime.Monday {
		sb.WriteString("MO,")
	}
	if m.MeetingTime.Tuesday {
		sb.WriteString("TU,")
	}
	if m.MeetingTime.Wednesday {
		sb.WriteString("WE,")
	}
	if m.MeetingTime.Thursday {
		sb.WriteString("TH,")
	}
	if m.MeetingTime.Friday {
		sb.WriteString("FR")
	}

	return sb.String()
}

const layout = "01/02/2006"

// StartDay returns the start date of the meeting time as a time.Time object
// This is not cached and is parsed on each invocation. It may also panic without handling.
func (m *MeetingTimeResponse) StartDay() time.Time {
	t, err := time.Parse(layout, m.MeetingTime.StartDate)
	if err != nil {
		log.Fatal().Err(err).Str("raw", m.MeetingTime.StartDate).Msg("Cannot parse start date")
	}
	return t
}

// EndDay returns the end date of the meeting time as a time.Time object
// This is not cached and is parsed on each invocation. It may also panic without handling.
func (m *MeetingTimeResponse) EndDay() time.Time {
	t, err := time.Parse(layout, m.MeetingTime.EndDate)
	if err != nil {
		log.Fatal().Err(err).Str("raw", m.MeetingTime.EndDate).Msg("Cannot parse end date")
	}
	return t
}

// StartTime returns the start time of the meeting time as a NaiveTime object
// This is not cached and is parsed on each invocation. It may also panic without handling.
func (m *MeetingTimeResponse) StartTime() *NaiveTime {
	raw := m.MeetingTime.BeginTime
	if raw == "" {
		return nil
	}

	value, err := strconv.ParseUint(raw, 10, 32)
	if err != nil {
		log.Fatal().Err(err).Str("raw", raw).Msg("Cannot parse start time integer")
	}

	return ParseNaiveTime(value)
}

// EndTime returns the end time of the meeting time as a NaiveTime object
// This is not cached and is parsed on each invocation. It may also panic without handling.
func (m *MeetingTimeResponse) EndTime() *NaiveTime {
	raw := m.MeetingTime.EndTime
	if raw == "" {
		return nil
	}

	value, err := strconv.ParseUint(raw, 10, 32)
	if err != nil {
		log.Fatal().Err(err).Str("raw", raw).Msg("Cannot parse end time integer")
	}

	return ParseNaiveTime(value)
}

// Converts the meeting time to a string that satisfies the iCalendar RRule format
func (m *MeetingTimeResponse) RRule() string {
	sb := strings.Builder{}

	sb.WriteString("FREQ=WEEKLY;")
	sb.WriteString(fmt.Sprintf("UNTIL=%s;", m.EndDay().Format("20060102")))
	sb.WriteString(fmt.Sprintf("BYDAY=%s;", m.ByDay()))

	return sb.String()
}

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
	Data []struct {
		// A internal identifier not used outside of the Banner system
		Id int `json:"id"`
		// The internal identifier for the term this class is in (e.g. 202420)
		Term string `json:"term"`
		// The human-readable name of the term this class is in (e.g. Fall 2021)
		TermDesc string `json:"termDesc"`
		// The specific identifier that describes this individual course. CRNs are unique to a term. (TODO: Verify this is true)
		CourseReferenceNumber string `json:"courseReferenceNumber"`
		PartOfTerm            string `json:"partOfTerm"`
		CourseNumber          string `json:"courseNumber"`
		// The short acronym of the course subject (e.g. CS, AEPI)
		Subject string `json:"subject"`
		// The full name of the course subject (e.g. Computer Science, Academic English Program-Intl.)
		SubjectDescription string `json:"subjectDescription"`
		// The specific section of the course (e.g. 001, 002)
		SequenceNumber string `json:"sequenceNumber"`
		// The long name of the campus this course takes place at (e.g. Main Campus, Downtown Campus)
		CampusDescription       string `json:"campusDescription"`
		ScheduleTypeDescription string `json:"scheduleTypeDescription"`
		// The long name of the course (generally)
		CourseTitle string `json:"courseTitle"`
		CreditHours int    `json:"creditHours"`
		// The maximum number of students that can enroll in this course
		MaximumEnrollment int `json:"maximumEnrollment"`
		// The number of students currently enrolled in this course
		Enrollment int `json:"enrollment"`
		// The number of seats available in this course (MaximumEnrollment - Enrollment)
		SeatsAvailable int `json:"seatsAvailable"`
		// The number of students that could waitlist for this course
		WaitCapacity int `json:"waitCapacity"`
		// The number of students currently on the waitlist for this course
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
		// A combination of the subject and course number (e.g. subject=CS, courseNumber=3443 => "CS3443")
		SubjectCourse                  string  `json:"subjectCourse"`
		ReservedSeatSummary            *string `json:"reservedSeatSummary"`
		InstructionalMethod            string  `json:"instructionalMethod"`
		InstructionalMethodDescription string  `json:"instructionalMethodDescription"`
		SectionAttributes              []struct {
			Class                 string `json:"class"`
			Code                  string `json:"code"`
			CourseReferenceNumber string `json:"courseReferenceNumber"`
			Description           string `json:"description"`
			IsZtcAttribute        bool   `json:"isZTCAttribute"`
			TermCode              string `json:"termCode"`
		} `json:"sectionAttributes"`
		Faculty         []FacultyItem         `json:"faculty"`
		MeetingsFaculty []MeetingTimeResponse `json:"meetingsFaculty"`
	} `json:"data"`
}
