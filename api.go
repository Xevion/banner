package banner

import (
	"encoding/json"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"
)

func getCourseMeetingTime(term int, crn int) MeetingTimeResponse {
	url := buildURL("searchResults/getFacultyMeetingTimes", map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
	})

	// Build the request
	log.Printf("GET %s", url)
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		log.Fatal(err)
	}
	AddUserAgent(req)

	resp, err := client.Do(req)
	if err != nil {
		log.Fatal(err)
	}

	if !ContainsContentType(resp.Header.Get("Content-Type"), "application/json") {
		log.Fatalf("Response was not JSON: %s", resp.Header.Get("Content-Type"))
	}

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
