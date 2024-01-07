package main

import (
	"strconv"
	"strings"
	"time"

	"github.com/samber/lo"
)

type Query struct {
	Subject             *string
	Title               *string
	Keywords            *[]string
	OpenOnly            *bool
	TermPart            *[]string // e.g. [1, B6, 8, J]
	Campus              *[]string // e.g. [9, 1DT, 1LR]
	InstructionalMethod *[]string // e.g. [HB]
	Attributes          *[]string // e.g. [060, 010]
	Instructor          *[]uint64 // e.g. [27957, 27961]
	StartTime           *time.Duration
	EndTime             *time.Duration
	CreditHours         *Range
	CourseNumberRange   *Range
}

type Range struct {
	Low  int
	High int
}

// FormatTimeParameter formats a time.Duration into a tuple of strings
// This is mostly a private helper to keep the parameter formatting for both the start and end time consistent together
func FormatTimeParameter(d time.Duration) (string, string, string) {
	hourParameter, minuteParameter, meridiemParameter := "", "", ""

	hours := int64(d.Hours())
	minutes := int64(d.Minutes()) % 60

	minuteParameter = string(minutes)

	if hours >= 12 {
		hourParameter = "PM"

		// Exceptional case: 12PM = 12, 1PM = 1, 2PM = 2
		if hours >= 13 {
			hourParameter = string(hours - 12) // 13 - 12 = 1, 14 - 12 = 2
		} else {
			hourParameter = string(hours)
		}
	} else {
		meridiemParameter = "AM"
		hourParameter = string(hours)
	}

	return hourParameter, minuteParameter, meridiemParameter
}

func (q *Query) Paramify() map[string]string {
	params := map[string]string{}

	if q.Subject != nil {
		params["txt_subject"] = *q.Subject
	}

	if q.Title != nil {
		// Whitespace can prevent valid queries from succeeding
		params["txt_title"] = strings.TrimSpace(*q.Title)
	}

	if q.Keywords != nil {
		params["txt_keyword"] = strings.Join(*q.Keywords, " ")
	}

	if q.OpenOnly != nil {
		params["chk_open_only"] = "true"
	}

	if q.TermPart != nil {
		params["txt_partOfTerm"] = strings.Join(*q.TermPart, ",")
	}

	if q.Campus != nil {
		params["txt_campus"] = strings.Join(*q.Campus, ",")
	}

	if q.Attributes != nil {
		params["txt_attribute"] = strings.Join(*q.Attributes, ",")
	}

	if q.Instructor != nil {
		params["txt_instructor"] = strings.Join(lo.Map(*q.Instructor, func(i uint64, _ int) string {
			return strconv.FormatUint(i, 10)
		}), ",")
	}

	if q.StartTime != nil {
		hour, minute, meridiem := FormatTimeParameter(*q.StartTime)
		params["select_start_hour"] = hour
		params["select_start_min"] = minute
		params["select_start_ampm"] = meridiem
	}

	if q.EndTime != nil {
		hour, minute, meridiem := FormatTimeParameter(*q.EndTime)
		params["select_end_hour"] = hour
		params["select_end_min"] = minute
		params["select_end_ampm"] = meridiem
	}

	if q.CreditHours != nil {
		params["txt_credithourlow"] = string(q.CreditHours.Low)
		params["txt_credithourhigh"] = string(q.CreditHours.High)
	}

	if q.CourseNumberRange != nil {
		params["txt_course_number_range"] = string(q.CourseNumberRange.Low)
		params["txt_course_number_range_to"] = string(q.CourseNumberRange.High)
	}

	return params
}
