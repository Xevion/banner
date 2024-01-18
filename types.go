package main

import (
	"strconv"
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
		Id                             int     `json:"id"`
		Term                           string  `json:"term"`
		TermDesc                       string  `json:"termDesc"`
		CourseReferenceNumber          string  `json:"courseReferenceNumber"`
		PartOfTerm                     string  `json:"partOfTerm"`
		CourseNumber                   string  `json:"courseNumber"`
		Subject                        string  `json:"subject"`
		SubjectDescription             string  `json:"subjectDescription"`
		SequenceNumber                 string  `json:"sequenceNumber"`
		CampusDescription              string  `json:"campusDescription"`
		ScheduleTypeDescription        string  `json:"scheduleTypeDescription"`
		CourseTitle                    string  `json:"courseTitle"`
		CreditHours                    int     `json:"creditHours"`
		MaximumEnrollment              int     `json:"maximumEnrollment"`
		Enrollment                     int     `json:"enrollment"`
		SeatsAvailable                 int     `json:"seatsAvailable"`
		WaitCapacity                   int     `json:"waitCapacity"`
		WaitCount                      int     `json:"waitCount"`
		CrossList                      *string `json:"crossList"`
		CrossListCapacity              *int    `json:"crossListCapacity"`
		CrossListCount                 *int    `json:"crossListCount"`
		CrossListAvailable             *int    `json:"crossListAvailable"`
		CreditHourHigh                 *int    `json:"creditHourHigh"`
		CreditHourLow                  *int    `json:"creditHourLow"`
		CreditHourIndicator            *string `json:"creditHourIndicator"`
		OpenSection                    bool    `json:"openSection"`
		LinkIdentifier                 *string `json:"linkIdentifier"`
		IsSectionLinked                bool    `json:"isSectionLinked"`
		SubjectCourse                  string  `json:"subjectCourse"`
		ReservedSeatSummary            *string `json:"reservedSeatSummary"`
		InstructionalMethod            string  `json:"instructionalMethod"`
		InstructionalMethodDescription string  `json:"instructionalMethodDescription"`
		Faculty                        []FacultyItem
		MeetingsFaculty                []MeetingTimeResponse `json:"meetingsFaculty"`
	} `json:"data"`
}

type MeetingTimeFaculty struct {
	bannerId    int
	category    string
	displayName string
	email       string
	primary     bool
}

type PrettyMeetingTimeResponse struct {
	faculty                []MeetingTimeFaculty
	weekdays               map[time.Weekday]bool
	campus                 string
	campusDescription      string
	creditHours            int
	building               string
	buildingDescription    string
	room                   string
	timeStart              NaiveTime
	timeEnd                NaiveTime
	dateStart              time.Time
	dateEnd                time.Time
	hoursPerWeek           float32
	meetingScheduleType    string
	meetingType            string
	meetingTypeDescription string
}
