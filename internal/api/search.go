package api

import (
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/samber/lo"
)

const (
	paramSubject           = "txt_subject"
	paramTitle             = "txt_courseTitle"
	paramKeywords          = "txt_keywordlike"
	paramOpenOnly          = "chk_open_only"
	paramTermPart          = "txt_partOfTerm"
	paramCampus            = "txt_campus"
	paramAttributes        = "txt_attribute"
	paramInstructor        = "txt_instructor"
	paramStartTimeHour     = "select_start_hour"
	paramStartTimeMinute   = "select_start_min"
	paramStartTimeMeridiem = "select_start_ampm"
	paramEndTimeHour       = "select_end_hour"
	paramEndTimeMinute     = "select_end_min"
	paramEndTimeMeridiem   = "select_end_ampm"
	paramMinCredits        = "txt_credithourlow"
	paramMaxCredits        = "txt_credithourhigh"
	paramCourseNumberLow   = "txt_course_number_range"
	paramCourseNumberHigh  = "txt_course_number_range_to"
	paramOffset            = "pageOffset"
	paramMaxResults        = "pageMaxSize"
)

// Query represents a search query for courses.
// It is a builder that allows for chaining methods to construct a query.
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
	minCredits          *int
	maxCredits          *int
	offset              int
	maxResults          int
	courseNumberRange   *Range
}

// NewQuery creates a new Query with default values.
func NewQuery() *Query {
	return &Query{maxResults: 8, offset: 0}
}

// Subject sets the subject for the query.
func (q *Query) Subject(subject string) *Query {
	q.subject = &subject
	return q
}

// Title sets the title for the query.
func (q *Query) Title(title string) *Query {
	q.title = &title
	return q
}

// Keywords sets the keywords for the query.
func (q *Query) Keywords(keywords []string) *Query {
	q.keywords = &keywords
	return q
}

// Keyword adds a keyword to the query.
func (q *Query) Keyword(keyword string) *Query {
	if q.keywords == nil {
		q.keywords = &[]string{keyword}
	} else {
		*q.keywords = append(*q.keywords, keyword)
	}
	return q
}

// OpenOnly sets whether to search for open courses only.
func (q *Query) OpenOnly(openOnly bool) *Query {
	q.openOnly = &openOnly
	return q
}

// TermPart sets the term part for the query.
func (q *Query) TermPart(termPart []string) *Query {
	q.termPart = &termPart
	return q
}

// Campus sets the campuses for the query.
func (q *Query) Campus(campus []string) *Query {
	q.campus = &campus
	return q
}

// InstructionalMethod sets the instructional methods for the query.
func (q *Query) InstructionalMethod(instructionalMethod []string) *Query {
	q.instructionalMethod = &instructionalMethod
	return q
}

// Attributes sets the attributes for the query.
func (q *Query) Attributes(attributes []string) *Query {
	q.attributes = &attributes
	return q
}

// Instructor sets the instructors for the query.
func (q *Query) Instructor(instructor []uint64) *Query {
	q.instructor = &instructor
	return q
}

// StartTime sets the start time for the query.
func (q *Query) StartTime(startTime time.Duration) *Query {
	q.startTime = &startTime
	return q
}

// EndTime sets the end time for the query.
func (q *Query) EndTime(endTime time.Duration) *Query {
	q.endTime = &endTime
	return q
}

// Credits sets the credit range for the query.
func (q *Query) Credits(low int, high int) *Query {
	q.minCredits = &low
	q.maxCredits = &high
	return q
}

// MinCredits sets the minimum credits for the query.
func (q *Query) MinCredits(value int) *Query {
	q.minCredits = &value
	return q
}

// MaxCredits sets the maximum credits for the query.
func (q *Query) MaxCredits(value int) *Query {
	q.maxCredits = &value
	return q
}

// CourseNumbers sets the course number range for the query.
func (q *Query) CourseNumbers(low int, high int) *Query {
	q.courseNumberRange = &Range{low, high}
	return q
}

// Offset sets the offset for pagination.
func (q *Query) Offset(offset int) *Query {
	q.offset = offset
	return q
}

// MaxResults sets the maximum number of results to return.
func (q *Query) MaxResults(maxResults int) *Query {
	q.maxResults = maxResults
	return q
}

// Range represents a range of two integers.
type Range struct {
	Low  int
	High int
}

// FormatTimeParameter formats a time.Duration into a tuple of strings for use in a POST request.
// It returns the hour, minute, and meridiem (AM/PM) as separate strings.
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

// Paramify converts a Query into a map of parameters for a POST request.
// This function assumes each query key only appears once.
func (q *Query) Paramify() map[string]string {
	params := map[string]string{}

	if q.subject != nil {
		params[paramSubject] = *q.subject
	}

	if q.title != nil {
		// Whitespace can prevent valid queries from succeeding
		params[paramTitle] = strings.TrimSpace(*q.title)
	}

	if q.keywords != nil {
		params[paramKeywords] = strings.Join(*q.keywords, " ")
	}

	if q.openOnly != nil {
		params[paramOpenOnly] = "true"
	}

	if q.termPart != nil {
		params[paramTermPart] = strings.Join(*q.termPart, ",")
	}

	if q.campus != nil {
		params[paramCampus] = strings.Join(*q.campus, ",")
	}

	if q.attributes != nil {
		params[paramAttributes] = strings.Join(*q.attributes, ",")
	}

	if q.instructor != nil {
		params[paramInstructor] = strings.Join(lo.Map(*q.instructor, func(i uint64, _ int) string {
			return strconv.FormatUint(i, 10)
		}), ",")
	}

	if q.startTime != nil {
		hour, minute, meridiem := FormatTimeParameter(*q.startTime)
		params[paramStartTimeHour] = hour
		params[paramStartTimeMinute] = minute
		params[paramStartTimeMeridiem] = meridiem
	}

	if q.endTime != nil {
		hour, minute, meridiem := FormatTimeParameter(*q.endTime)
		params[paramEndTimeHour] = hour
		params[paramEndTimeMinute] = minute
		params[paramEndTimeMeridiem] = meridiem
	}

	if q.minCredits != nil {
		params[paramMinCredits] = strconv.Itoa(*q.minCredits)
	}

	if q.maxCredits != nil {
		params[paramMaxCredits] = strconv.Itoa(*q.maxCredits)
	}

	if q.courseNumberRange != nil {
		params[paramCourseNumberLow] = strconv.Itoa(q.courseNumberRange.Low)
		params[paramCourseNumberHigh] = strconv.Itoa(q.courseNumberRange.High)
	}

	params[paramOffset] = strconv.Itoa(q.offset)
	params[paramMaxResults] = strconv.Itoa(q.maxResults)

	return params
}

// String returns a string representation of the query, ideal for debugging & logging.
func (q *Query) String() string {
	var sb strings.Builder

	if q.subject != nil {
		fmt.Fprintf(&sb, "subject=%s, ", *q.subject)
	}

	if q.title != nil {
		// Whitespace can prevent valid queries from succeeding
		fmt.Fprintf(&sb, "title=%s, ", strings.TrimSpace(*q.title))
	}

	if q.keywords != nil {
		fmt.Fprintf(&sb, "keywords=%s, ", strings.Join(*q.keywords, " "))
	}

	if q.openOnly != nil {
		fmt.Fprintf(&sb, "openOnly=%t, ", *q.openOnly)
	}

	if q.termPart != nil {
		fmt.Fprintf(&sb, "termPart=%s, ", strings.Join(*q.termPart, ","))
	}

	if q.campus != nil {
		fmt.Fprintf(&sb, "campus=%s, ", strings.Join(*q.campus, ","))
	}

	if q.attributes != nil {
		fmt.Fprintf(&sb, "attributes=%s, ", strings.Join(*q.attributes, ","))
	}

	if q.instructor != nil {
		fmt.Fprintf(&sb, "instructor=%s, ", strings.Join(lo.Map(*q.instructor, func(i uint64, _ int) string {
			return strconv.FormatUint(i, 10)
		}), ","))
	}

	if q.startTime != nil {
		hour, minute, meridiem := FormatTimeParameter(*q.startTime)
		fmt.Fprintf(&sb, "startTime=%s:%s%s, ", hour, minute, meridiem)
	}

	if q.endTime != nil {
		hour, minute, meridiem := FormatTimeParameter(*q.endTime)
		fmt.Fprintf(&sb, "endTime=%s:%s%s, ", hour, minute, meridiem)
	}

	if q.minCredits != nil {
		fmt.Fprintf(&sb, "minCredits=%d, ", *q.minCredits)
	}

	if q.maxCredits != nil {
		fmt.Fprintf(&sb, "maxCredits=%d, ", *q.maxCredits)
	}

	if q.courseNumberRange != nil {
		fmt.Fprintf(&sb, "courseNumberRange=%d-%d, ", q.courseNumberRange.Low, q.courseNumberRange.High)
	}

	fmt.Fprintf(&sb, "offset=%d, ", q.offset)
	fmt.Fprintf(&sb, "maxResults=%d", q.maxResults)

	return sb.String()
}

// Dict returns a map representation of the query, ideal for debugging & logging.
// This dict is represented with zerolog's Event type.
// func (q *Query) Dict() *zerolog.Event {
// }
