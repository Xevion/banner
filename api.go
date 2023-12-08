package banner

import (
	"bytes"
	"encoding/json"
	"log"
	"strconv"
	"strings"
	"time"
)

var (
	sessionID string
)

func init() {
	sessionID = RandomString(5) + strconv.Itoa(int(time.Now().UnixMilli()))
	log.Printf("Session ID: %s", sessionID)
}

// GET /classSearch/getTerms?searchTerm=&offset=1&max=10&_=1702069154094
func GetTerms() {
	req := BuildRequest("GET", "classSearch/getTerms", map[string]string{
		"searchTerm": "",
		"offset":     "1",
		"max":        "10",
		"_":          strconv.Itoa(int(time.Now().UnixMilli())),
	})
}

// GET /classSearch/get_partOfTerm?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1702070282288
func GetPartOfTerms() {
	req := BuildRequest("GET", "classSearch/get_partOfTerm", map[string]string{
		"searchTerm":      "",
		"term":            "202420",
		"offset":          "1",
		"max":             "10",
		"uniqueSessionId": sessionID,
		"_":               strconv.Itoa(int(time.Now().UnixMilli())),
	})
}

// GET /classSearch/get_instructor?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1701951338584
func GetInstructor() {
	req := BuildRequest("GET", "classSearch/get_instructor", map[string]string{
		"searchTerm":      "",
		"term":            "202420",
		"offset":          "1",
		"max":             "10",
		"uniqueSessionId": sessionID,
		"_":               strconv.Itoa(int(time.Now().UnixMilli())),
	})
}

func GetClassDetails() {
	body, _ := json.Marshal(map[string]string{
		"term":                  "202420",
		"courseReferenceNumber": "36522",
		"first":                 "first",
	})
	req := BuildRequestWithBody("GET", "searchResults/getClassDetails", nil, bytes.NewBuffer(body))
}

// GET /searchResults/searchResults?txt_instructor=77521&txt_term=202420&startDatepicker=&endDatepicker=&uniqueSessionId=4bzai1701944879219&pageOffset=0&pageMaxSize=10&sortColumn=subjectDescription&sortDirection=asc
// GET /searchResults/searchResults?txt_subject=CS&txt_keywordlike=Application&txt_term=202420&startDatepicker=&endDatepicker=&uniqueSessionId=4bzai1701944879219&pageOffset=0&pageMaxSize=10&sortColumn=subjectDescription&sortDirection=asc
func Search() {
	req := BuildRequest("GET", "classSearch/get_subject", map[string]string{
		"txt_subject":     "CS",
		"txt_keywordlike": "Application",
		"txt_term":        "202420",
		"startDatepicker": "",
		"endDatepicker":   "",
		"uniqueSessionId": sessionID,
		"pageOffset":      "0",
		"pageMaxSize":     "10",
		"sortColumn":      "subjectDescription",
		"sortDirection":   "asc",
	})
}

// GET /classSearch/get_subject?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1702069787420
func GetSubjects() {
	req := BuildRequest("GET", "classSearch/get_subject", map[string]string{
		"searchTerm":      "",
		"term":            "202420",
		"offset":          "1",
		"max":             "10",
		"uniqueSessionId": sessionID,
		"_":               "1702069787420",
	})
}

// /classSearch/get_campus?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1702070341071
func GetCampuses() {
	req := BuildRequest("GET", "classSearch/get_campus", map[string]string{
		"searchTerm":      "",
		"term":            "202420",
		"offset":          "1",
		"max":             "10",
		"uniqueSessionId": sessionID,
		"_":               strconv.Itoa(int(time.Now().UnixMilli())),
	})
}

// / classSearch/get_instructionalMethod?searchTerm=&term=202420&offset=1&max=10&uniqueSessionId=4bzai1701944879219&_=1702070364082
func GetInstructionalMethods() {
	req := BuildRequest("GET", "classSearch/get_instructionalMethod", map[string]string{
		"searchTerm":      "",
		"term":            "202420",
		"offset":          "1",
		"max":             "10",
		"uniqueSessionId": sessionID,
		"_":               strconv.Itoa(int(time.Now().UnixMilli())),
	})
}

// GetCourseMeetingTime retrieves the meeting time information for a course based on the given term and course reference number (CRN).
// It makes an HTTP GET request to the appropriate API endpoint and parses the response to extract the meeting time data.
// The function returns a MeetingTimeResponse struct containing the extracted information.
func GetCourseMeetingTime(term int, crn int) MeetingTimeResponse {
	req := BuildRequest("GET", "searchResults/getFacultyMeetingTimes", map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
	})

	log.Printf("GET %s", req.URL.String())
	resp, err := client.Do(req)
	if err != nil {
		log.Fatal(err)
	}

	// Assert that the response is JSON
	if !ContainsContentType(resp.Header.Get("Content-Type"), "application/json") {
		log.Fatalf("Response was not JSON: %s", resp.Header.Get("Content-Type"))
	}

	// Read the response body into JSON
	defer resp.Body.Close()
	var body map[string][]map[string]interface{}
	err = json.NewDecoder(resp.Body).Decode(&body)
	if err != nil {
		log.Fatal(err)
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
	timeStart, _ := strconv.ParseUint(meetingTimeMap["beginTime"].(string), 10, 0)
	timeEnd, _ := strconv.ParseUint(meetingTimeMap["endTime"].(string), 10, 0)

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

	return MeetingTimeResponse{
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
	}
}
