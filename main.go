package banner

import (
	"log"
	"net/http"
	"net/http/cookiejar"
	"os"
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

	setup()

	meetingTime := getCourseMeetingTime(202420, 44142)
	log.Println(meetingTime)
}
