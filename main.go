package main

import (
	"encoding/json"
	"log"
	"net/http"
	"net/http/cookiejar"
	"net/url"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/joho/godotenv"
)

var (
	// Base URL for all requests to the banner system
	baseURL string
	client  http.Client
	cookies http.CookieJar
)

type MeetingTimeFaculty struct {
	bannerId    int
	category    string
	displayName string
	email       string
	primary     bool
}

type MeetingTimeResponse struct {
	faculty                []MeetingTimeFaculty
	weekdays               map[time.Weekday]bool
	campus                 string
	campusDescription      string
	creditHours            int
	building               string
	buildingDescription    string
	room                   string
	timeStart              uint64
	timeEnd                uint64
	dateStart              time.Time
	dateEnd                time.Time
	hoursPerWeek           float32
	meetingScheduleType    string
	meetingType            string
	meetingTypeDescription string
}

func setupSession() {
	// Makes the initial requests that sets up the session cookies for the rest of the application
	log.Println("Setting up session...")

	request_queue := []string{
		"/registration/registration",
		"/selfServiceMenu/data",
	}

	for _, path := range request_queue {
		req, _ := http.NewRequest("GET", buildURL(path, nil), nil)
		AddUserAgent(req)
		res, _ := client.Do(req)
		log.Println(res)
	}

	// Validate that cookies were set
	baseURL_parsed, _ := url.Parse(baseURL)
	current_cookies := cookies.Cookies(baseURL_parsed)
	required_cookies := map[string]bool{
		"JSESSIONID": false,
		"SSB_COOKIE": false,
	}

	for _, cookie := range current_cookies {
		_, present := required_cookies[cookie.Name]
		// Check if this cookie is required
		if present {
			required_cookies[cookie.Name] = true
		}
	}

	// Check if all required cookies were set
	for cookie_name, cookie_set := range required_cookies {
		if !cookie_set {
			log.Fatalf("Required cookie %s was not set", cookie_name)
		}
	}
	log.Println("All cookies acquired. Session setup complete.")

	// Validate that the session allows access to termSelection
}

func buildURL(path string, params map[string]string) string {
	// Builds a URL for the given path and parameters
	url := baseURL + path

	if params != nil {
		takenFirst := false
		for key, value := range params {
			paramChar := "&"
			if !takenFirst {
				paramChar = "?"
				takenFirst = true
			}

			url += paramChar + key + "=" + value
		}
	}

	return url
}

func AddUserAgent(req *http.Request) {
	req.Header.Add("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
}

func ContainsContentType(header string, search string) bool {
	// Split on commas, check if any of the types match
	for _, content_type := range strings.Split(header, ";") {
		if content_type == search {
			return true
		}
	}
	return false
}

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

func main() {
	err := godotenv.Load()
	if err != nil {
		log.Fatal("Error loading .env file")
	}

	baseURL = os.Getenv("BANNER_BASE_URL")
	cookies, err = cookiejar.New(nil)
	if err != nil {
		log.Fatal(err)
	}

	client = http.Client{
		Jar: cookies,
	}

	setupSession()

	meetingTime := getCourseMeetingTime(202420, 44142)
	log.Println(meetingTime)
}
