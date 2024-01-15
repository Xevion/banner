package main

import (
	"bytes"
	"encoding/json"
	"io"
	"strconv"
	"strings"
	"time"

	log "github.com/rs/zerolog/log"
)

var sessionID string = RandomString(5) + Nonce()

// BannerTerm represents a term in the Banner system
type BannerTerm struct {
	Code        string `json:"code"`
	Description string `json:"description"`
}

// GET /classSearch/getTerms?searchTerm=&offset=1&max=10&_=1702069154094
func GetTerms(search string, offset int, max int) ([]BannerTerm, error) {
	req := BuildRequest("GET", "/classSearch/getTerms", map[string]string{
		"searchTerm": search,
		"offset":     strconv.Itoa(offset),
		"max":        strconv.Itoa(max),
		"_":          Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil, err
	}

	// Assert that the response is JSON
	if contentType := res.Header.Get("Content-Type"); !strings.Contains(contentType, JsonContentType) {
		return nil, &UnexpectedContentTypeError{
			Expected: JsonContentType,
			Actual:   contentType,
		}
	}

	// print the response body
	defer res.Body.Close()
	body, _ := io.ReadAll(res.Body)

	terms := make([]BannerTerm, 0, 10)
	json.Unmarshal(body, &terms)

	if err != nil {
		return nil, err
	}

	return terms, nil
}

// GET /classSearch/get_partOfTerm?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1702070282288
func GetPartOfTerms(search string, term int, offset int, max int) ([]BannerTerm, error) {
	req := BuildRequest("GET", "/classSearch/get_partOfTerm", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil, err
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Printf("ERR Response was not JSON: %s", res.Header.Get("Content-Type"))
	}

	// print the response body
	defer res.Body.Close()
	body, _ := io.ReadAll(res.Body)

	terms := make([]BannerTerm, 0, 10)
	err = json.Unmarshal(body, &terms)
	if err != nil {
		return nil, err
	}

	return terms, nil
}

type Instructor struct{}

// GET /classSearch/get_instructor?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1701951338584
func GetInstructor(search string, term int, offset int, max int) []Instructor {
	req := BuildRequest("GET", "/classSearch/get_instructor", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Printf("ERR Response was not JSON: %s", res.Header.Get("Content-Type"))
	}

	return make([]Instructor, 0)
}

type ClassDetails struct{}

func GetClassDetails(term int, crn int) *ClassDetails {
	body, _ := json.Marshal(map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
		"first":                 "first", // TODO: What is this?
	})
	req := BuildRequestWithBody("GET", "/searchResults/getClassDetails", nil, bytes.NewBuffer(body))

	res, err := DoRequest(req)
	if err != nil {
		return nil
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Printf("ERR Response was not JSON: %s", res.Header.Get("Content-Type"))
	}

	return &ClassDetails{}
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
		Id                      int     `json:"id"`
		Term                    string  `json:"term"`
		TermDesc                string  `json:"termDesc"`
		CourseReferenceNumber   string  `json:"courseReferenceNumber"`
		PartOfTerm              string  `json:"partOfTerm"`
		CourseNumber            string  `json:"courseNumber"`
		Subject                 string  `json:"subject"`
		SubjectDescription      string  `json:"subjectDescription"`
		SequenceNumber          string  `json:"sequenceNumber"`
		CampusDescription       string  `json:"campusDescription"`
		ScheduleTypeDescription string  `json:"scheduleTypeDescription"`
		CourseTitle             string  `json:"courseTitle"`
		CreditHours             int     `json:"creditHours"`
		MaximumEnrollment       int     `json:"maximumEnrollment"`
		Enrollment              int     `json:"enrollment"`
		SeatsAvailable          int     `json:"seatsAvailable"`
		WaitCapacity            int     `json:"waitCapacity"`
		WaitCount               int     `json:"waitCount"`
		CrossList               *string `json:"crossList"`
		CrossListCapacity       *int    `json:"crossListCapacity"`
	} `json:"data"`
}

// GET /searchResults/searchResults?txt_instructor=77521&txt_term=202420&startDatepicker=&endDatepicker=&uniqueSessionId=4bzai1701944879219&pageOffset=0&pageMaxSize=10&sortColumn=subjectDescription&sortDirection=asc
// GET /searchResults/searchResults?txt_subject=CS&txt_keywordlike=Application&txt_term=202420&startDatepicker=&endDatepicker=&uniqueSessionId=4bzai1701944879219&pageOffset=0&pageMaxSize=10&sortColumn=subjectDescription&sortDirection=asc
func Search(query Query, offset int, max int, sort string, sortDescending bool) (*SearchResult, error) {
	params := query.Paramify()

	params["txt_term"] = "202420" // TODO: Make this automatic but dynamically specifiable
	params["uniqueSessionId"] = sessionID
	params["pageOffset"] = strconv.Itoa(offset)
	params["pageMaxSize"] = strconv.Itoa(max)
	params["sortColumn"] = sort
	params["sortDirection"] = "asc"
	// These dates are not available for usage anywhere in the UI, but are included in every query
	params["startDatepicker"] = ""
	params["endDatepicker"] = ""

	req := BuildRequest("GET", "/searchResults/searchResults", params)

	res, err := DoRequest(req)
	if err != nil {
		return nil, err
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Error().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	body, _ := io.ReadAll(res.Body)

	var result SearchResult
	err = json.Unmarshal(body, &result)

	if err != nil {
		return nil, err
	}

	return &result, nil
}

type Subject struct{}

// GET /classSearch/get_subject?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1702069787420
func GetSubjects(search string, term int, offset int, max int) []Subject {
	req := BuildRequest("GET", "/classSearch/get_subject", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Printf("ERR Response was not JSON: %s", res.Header.Get("Content-Type"))
	}

	return make([]Subject, 0)
}

type Campus struct{}

// /classSearch/get_campus?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1702070341071
func GetCampuses(search string, term int, offset int, max int) []Campus {
	req := BuildRequest("GET", "/classSearch/get_campus", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		log.Printf("ERR %s", err)
		return nil
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Printf("ERR Response was not JSON: %s", res.Header.Get("Content-Type"))
	}

	return make([]Campus, 0)
}

type InstructionalMethod struct{}

// / classSearch/get_instructionalMethod?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1702070364082
func GetInstructionalMethods(search string, term int, offset int, max int) ([]InstructionalMethod, error) {
	req := BuildRequest("GET", "/classSearch/get_instructionalMethod", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		log.Printf("ERR %s", err)
		return nil, err
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Printf("ERR Response was not JSON: %s", res.Header.Get("Content-Type"))
	}

	return make([]InstructionalMethod, 0), nil
}

// GetCourseMeetingTime retrieves the meeting time information for a course based on the given term and course reference number (CRN).
// It makes an HTTP GET request to the appropriate API endpoint and parses the response to extract the meeting time data.
// The function returns a MeetingTimeResponse struct containing the extracted information.
func GetCourseMeetingTime(term int, crn int) (*MeetingTimeResponse, error) {
	req := BuildRequest("GET", "/searchResults/getFacultyMeetingTimes", map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil, err
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Msgf("Response was not JSON: %s", res.Header.Get("Content-Type"))
	}

	// Read the response body into JSON
	defer res.Body.Close()
	var body map[string][]map[string]interface{}
	err = json.NewDecoder(res.Body).Decode(&body)
	if err != nil {
		return nil, err
	}

	if len(body["fmt"]) > 1 {
		log.Printf("Expected only one fmt child, found %d", len(body["fmt"]))
	}

	// Acquire & cast the meeting time data
	meetingTimeMap := body["fmt"][0]["meetingTime"].(map[string]interface{})

	// Extract weekdays
	weekdays := make(map[time.Weekday]bool)
	for i := int(time.Sunday); i <= int(time.Saturday); i++ {
		day := time.Weekday(i)
		dayActive := meetingTimeMap[strings.ToLower(day.String())].(bool)
		weekdays[day] = dayActive
	}

	meetingScheduleType := meetingTimeMap["meetingScheduleType"].(string)
	meetingType := meetingTimeMap["meetingType"].(string)
	meetingTypeDescription := meetingTimeMap["meetingTypeDescription"].(string)

	// Extract other data
	campus := meetingTimeMap["campus"].(string)
	campusDescription := meetingTimeMap["campusDescription"].(string)
	building := meetingTimeMap["building"].(string)
	buildingDescription := meetingTimeMap["buildingDescription"].(string)
	room := "N/A"
	if meetingTimeMap["room"] != nil {
		room = meetingTimeMap["room"].(string)
	}

	creditHours := meetingTimeMap["creditHourSession"].(float64)
	hoursPerWeek := meetingTimeMap["hoursWeek"].(float64)

	// Parse dates & times
	const layout = "01/02/2006"
	dateStart, _ := time.Parse(layout, meetingTimeMap["startDate"].(string))
	dateEnd, _ := time.Parse(layout, meetingTimeMap["endDate"].(string))
	timeStartInt, _ := strconv.ParseUint(meetingTimeMap["beginTime"].(string), 10, 0)
	timeStart := ParseNaiveTime(timeStartInt)
	timeEndInt, _ := strconv.ParseUint(meetingTimeMap["endTime"].(string), 10, 0)
	timeEnd := ParseNaiveTime(timeEndInt)

	// Extract faculty data
	faculty := make([]MeetingTimeFaculty, 0)
	for _, facultyMap := range body["fmt"][0]["faculty"].([]interface{}) {
		facultyMap := facultyMap.(map[string]interface{})
		bannerId, _ := strconv.Atoi(facultyMap["bannerId"].(string))
		faculty = append(faculty, MeetingTimeFaculty{
			bannerId:    bannerId,
			category:    facultyMap["category"].(string),
			displayName: facultyMap["displayName"].(string),
			email:       facultyMap["emailAddress"].(string),
			primary:     facultyMap["primaryIndicator"].(bool),
		})
	}

	return &MeetingTimeResponse{
		faculty:                faculty,
		weekdays:               weekdays,
		campus:                 campus,
		campusDescription:      campusDescription,
		creditHours:            int(creditHours),
		building:               building,
		buildingDescription:    buildingDescription,
		room:                   room,
		timeStart:              timeStart,
		timeEnd:                timeEnd,
		dateStart:              dateStart,
		dateEnd:                dateEnd,
		hoursPerWeek:           float32(hoursPerWeek),
		meetingScheduleType:    meetingScheduleType,
		meetingType:            meetingType,
		meetingTypeDescription: meetingTypeDescription,
	}, nil
}
