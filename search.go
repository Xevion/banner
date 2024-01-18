package main

import (
	"strconv"
	"strings"
	"time"

	"github.com/samber/lo"
)

type Query struct {
	subject             *string
	title               *string
	keywords            *[]string
	openOnly            *bool
	termPart            *[]string // e.g. [1, B6, 8, J]
	campus              *[]string // e.g. [9, 1DT, 1LR]
	instructionalMethod *[]string // e.g. [HB]
	attributes          *[]string // e.g. [060, 010]
	instructor          *[]uint64 // e.g. [27957, 27961]
	startTime           *time.Duration
	endTime             *time.Duration
	creditHours         *Range
	courseNumberRange   *Range
}

func NewQuery() *Query {
	return &Query{}
}

// Subject sets the subject for the query
func (q *Query) Subject(subject string) *Query {
	q.subject = &subject
	return q
}

// Title sets the title for the query
func (q *Query) Title(title string) *Query {
	q.title = &title
	return q
}

// Keywords sets the keywords for the query
func (q *Query) Keywords(keywords []string) *Query {
	q.keywords = &keywords
	return q
}

// Keyword adds a keyword to the query
func (q *Query) Keyword(keyword string) *Query {
	if q.keywords == nil {
		q.keywords = &[]string{keyword}
	} else {
		*q.keywords = append(*q.keywords, keyword)
	}
	return q
}

// OpenOnly sets the open only flag for the query
func (q *Query) OpenOnly(openOnly bool) *Query {
	q.openOnly = &openOnly
	return q
}

// TermPart sets the term part for the query
func (q *Query) TermPart(termPart []string) *Query {
	q.termPart = &termPart
	return q
}

func (q *Query) Campus(campus []string) *Query {
	q.campus = &campus
	return q
}

func (q *Query) InstructionalMethod(instructionalMethod []string) *Query {
	q.instructionalMethod = &instructionalMethod
	return q
}

func (q *Query) Attributes(attributes []string) *Query {
	q.attributes = &attributes
	return q
}

func (q *Query) Instructor(instructor []uint64) *Query {
	q.instructor = &instructor
	return q
}

func (q *Query) StartTime(startTime time.Duration) *Query {
	q.startTime = &startTime
	return q
}

func (q *Query) EndTime(endTime time.Duration) *Query {
	q.endTime = &endTime
	return q
}

func (q *Query) CreditHours(creditHours Range) *Query {
	q.creditHours = &creditHours
	return q
}

func (q *Query) CourseNumberRange(courseNumberRange Range) *Query {
	q.courseNumberRange = &courseNumberRange
	return q
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

	minuteParameter = strconv.FormatInt(minutes, 10)

	if hours >= 12 {
		hourParameter = "PM"

		// Exceptional case: 12PM = 12, 1PM = 1, 2PM = 2
		if hours >= 13 {
			hourParameter = strconv.FormatInt(hours-12, 10) // 13 - 12 = 1, 14 - 12 = 2
		} else {
			hourParameter = strconv.FormatInt(hours, 10)
		}
	} else {
		meridiemParameter = "AM"
		hourParameter = strconv.FormatInt(hours, 10)
	}

	return hourParameter, minuteParameter, meridiemParameter
}

// Paramify converts a Query into a map of parameters that can be used in a POST request
// This function assumes each query key only appears once.
func (q *Query) Paramify() map[string]string {
	params := map[string]string{}

	if q.subject != nil {
		params["txt_subject"] = *q.subject
	}

	if q.title != nil {
		// Whitespace can prevent valid queries from succeeding
		params["txt_title"] = strings.TrimSpace(*q.title)
	}

	if q.keywords != nil {
		params["txt_keyword"] = strings.Join(*q.keywords, " ")
	}

	if q.openOnly != nil {
		params["chk_open_only"] = "true"
	}

	if q.termPart != nil {
		params["txt_partOfTerm"] = strings.Join(*q.termPart, ",")
	}

	if q.campus != nil {
		params["txt_campus"] = strings.Join(*q.campus, ",")
	}

	if q.attributes != nil {
		params["txt_attribute"] = strings.Join(*q.attributes, ",")
	}

	if q.instructor != nil {
		params["txt_instructor"] = strings.Join(lo.Map(*q.instructor, func(i uint64, _ int) string {
			return strconv.FormatUint(i, 10)
		}), ",")
	}

	if q.startTime != nil {
		hour, minute, meridiem := FormatTimeParameter(*q.startTime)
		params["select_start_hour"] = hour
		params["select_start_min"] = minute
		params["select_start_ampm"] = meridiem
	}

	if q.endTime != nil {
		hour, minute, meridiem := FormatTimeParameter(*q.endTime)
		params["select_end_hour"] = hour
		params["select_end_min"] = minute
		params["select_end_ampm"] = meridiem
	}

	if q.creditHours != nil {
		params["txt_credithourlow"] = strconv.Itoa(q.creditHours.Low)
		params["txt_credithourhigh"] = strconv.Itoa(q.creditHours.High)
	}

	if q.courseNumberRange != nil {
		params["txt_course_number_range"] = strconv.Itoa(q.courseNumberRange.Low)
		params["txt_course_number_range_to"] = strconv.Itoa(q.courseNumberRange.High)
	}

	return params
}
