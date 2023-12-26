package main

import "time"

const JsonContentType = "application/json"

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
	timeStart              NaiveTime
	timeEnd                NaiveTime
	dateStart              time.Time
	dateEnd                time.Time
	hoursPerWeek           float32
	meetingScheduleType    string
	meetingType            string
	meetingTypeDescription string
}
