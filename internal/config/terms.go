package config

import (
	"fmt"
	"strconv"
	"time"
)

// Term selection should yield smart results based on the current time, as well as the input provided.
// Fall 2024, "spring" => Spring 2025
// Fall 2024, "fall" => Fall 2025
// Summer 2024, "fall" => Fall 2024

const (
	// Fall is the first term of the school year.
	Fall = iota
	// Spring is the second term of the school year.
	Spring
	// Summer is the third term of the school year.
	Summer
)

// Term represents a school term, consisting of a year and a season.
type Term struct {
	Year   uint16
	Season uint8
}

// SeasonRanges represents the start and end day of each term within a year.
type SeasonRanges struct {
	Spring YearDayRange
	Summer YearDayRange
	Fall   YearDayRange
}

// YearDayRange represents the start and end day of a term within a year.
type YearDayRange struct {
	Start uint16
	End   uint16
}

// GetYearDayRange returns the start and end day of each term for the given year.
// The ranges are inclusive of the start day and exclusive of the end day.
func GetYearDayRange(loc *time.Location, year uint16) SeasonRanges {
	springStart := time.Date(int(year), time.January, 14, 0, 0, 0, 0, loc).YearDay()
	springEnd := time.Date(int(year), time.May, 1, 0, 0, 0, 0, loc).YearDay()
	summerStart := time.Date(int(year), time.May, 25, 0, 0, 0, 0, loc).YearDay()
	summerEnd := time.Date(int(year), time.August, 15, 0, 0, 0, 0, loc).YearDay()
	fallStart := time.Date(int(year), time.August, 18, 0, 0, 0, 0, loc).YearDay()
	fallEnd := time.Date(int(year), time.December, 10, 0, 0, 0, 0, loc).YearDay()

	return SeasonRanges{
		Spring: YearDayRange{
			Start: uint16(springStart),
			End:   uint16(springEnd),
		},
		Summer: YearDayRange{
			Start: uint16(summerStart),
			End:   uint16(summerEnd),
		},
		Fall: YearDayRange{
			Start: uint16(fallStart),
			End:   uint16(fallEnd),
		},
	}
}

// GetCurrentTerm returns the current and next terms based on the provided time.
// The current term can be nil if the time falls between terms.
// The 'year' in the term corresponds to the academic year, which may differ from the calendar year.
func GetCurrentTerm(ranges SeasonRanges, now time.Time) (*Term, *Term) {
	literalYear := uint16(now.Year())
	dayOfYear := uint16(now.YearDay())

	// If we're past the end of the summer term, we're 'in' the next school year.
	var termYear uint16
	if dayOfYear > ranges.Summer.End {
		termYear = literalYear + 1
	} else {
		termYear = literalYear
	}

	if (dayOfYear < ranges.Spring.Start) || (dayOfYear >= ranges.Fall.End) {
		// Fall over, Spring not yet begun
		return nil, &Term{Year: termYear, Season: Spring}
	} else if (dayOfYear >= ranges.Spring.Start) && (dayOfYear < ranges.Spring.End) {
		// Spring
		return &Term{Year: termYear, Season: Spring}, &Term{Year: termYear, Season: Summer}
	} else if dayOfYear < ranges.Summer.Start {
		// Spring over, Summer not yet begun
		return nil, &Term{Year: termYear, Season: Summer}
	} else if (dayOfYear >= ranges.Summer.Start) && (dayOfYear < ranges.Summer.End) {
		// Summer
		return &Term{Year: termYear, Season: Summer}, &Term{Year: termYear, Season: Fall}
	} else if dayOfYear < ranges.Fall.Start {
		// Summer over, Fall not yet begun
		return nil, &Term{Year: termYear, Season: Fall}
	} else if (dayOfYear >= ranges.Fall.Start) && (dayOfYear < ranges.Fall.End) {
		// Fall
		return &Term{Year: termYear, Season: Fall}, nil
	}

	panic(fmt.Sprintf("Impossible Code Reached (dayOfYear: %d)", dayOfYear))
}

// ParseTerm converts a Banner term code string to a Term struct.
func ParseTerm(code string) Term {
	year, _ := strconv.ParseUint(code[0:4], 10, 16)

	var season uint8
	termCode := code[4:6]
	switch termCode {
	case "10":
		season = Fall
	case "20":
		season = Spring
	case "30":
		season = Summer
	}

	return Term{
		Year:   uint16(year),
		Season: season,
	}
}

// ToString converts a Term struct to a Banner term code string.
func (term Term) ToString() string {
	var season string
	switch term.Season {
	case Fall:
		season = "10"
	case Spring:
		season = "20"
	case Summer:
		season = "30"
	}

	return fmt.Sprintf("%d%s", term.Year, season)
}
