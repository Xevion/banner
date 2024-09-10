package main

import (
	"fmt"
	"strconv"
	"time"

	"github.com/rs/zerolog/log"
)

// Term selection should yield smart results based on the current time, as well as the input provided.
// Fall 2024, "spring" => Spring 2025
// Fall 2024, "fall" => Fall 2025
// Summer 2024, "fall" => Fall 2024

const (
	Spring = iota
	Summer
	Fall
)

type Term struct {
	Year   uint16
	Season uint8
}

var (
	SpringRange, SummerRange, FallRange YearDayRange
)

func init() {
	SpringRange, SummerRange, FallRange = GetYearDayRange(uint16(time.Now().Year()))

	currentTerm, nextTerm := GetCurrentTerm(time.Now())
	log.Debug().Str("CurrentTerm", fmt.Sprintf("%+v", currentTerm)).Str("NextTerm", fmt.Sprintf("%+v", nextTerm)).Msg("GetCurrentTerm")
}

type YearDayRange struct {
	Start uint16
	End   uint16
}

// GetYearDayRange returns the start and end day of each term for the given year.
// This could technically introduce race conditions, but it's more likely confusion from UTC will be a greater issue.
// Spring: January 14th to May
// Summer: May 25th - August 15th
// Fall: August 18th - December 10th
func GetYearDayRange(year uint16) (YearDayRange, YearDayRange, YearDayRange) {
	springStart := time.Date(int(year), time.January, 14, 0, 0, 0, 0, CentralTimeLocation).YearDay()
	springEnd := time.Date(int(year), time.May, 1, 0, 0, 0, 0, CentralTimeLocation).YearDay()
	summerStart := time.Date(int(year), time.May, 25, 0, 0, 0, 0, CentralTimeLocation).YearDay()
	summerEnd := time.Date(int(year), time.August, 15, 0, 0, 0, 0, CentralTimeLocation).YearDay()
	fallStart := time.Date(int(year), time.August, 18, 0, 0, 0, 0, CentralTimeLocation).YearDay()
	fallEnd := time.Date(int(year), time.December, 10, 0, 0, 0, 0, CentralTimeLocation).YearDay()

	return YearDayRange{
			Start: uint16(springStart),
			End:   uint16(springEnd),
		}, YearDayRange{
			Start: uint16(summerStart),
			End:   uint16(summerEnd),
		}, YearDayRange{
			Start: uint16(fallStart),
			End:   uint16(fallEnd),
		}
}

// GetCurrentTerm returns the current term, and the next term. Only the first term is nillable.
// YearDay ranges are inclusive of the start, and exclusive of the end.
func GetCurrentTerm(now time.Time) (*Term, *Term) {
	year := uint16(now.Year())
	dayOfYear := uint16(now.YearDay())

	// Fall of 2024 => 202410
	// Spring of 2024 => 202420
	// Fall of 2025 => 202510
	// Summer of 2025 => 202530

	if (dayOfYear < SpringRange.Start) || (dayOfYear >= FallRange.End) {
		// Fall over, Spring not yet begun
		return nil, &Term{Year: year + 1, Season: Spring}
	} else if (dayOfYear >= SpringRange.Start) && (dayOfYear < SpringRange.End) {
		// Spring
		return &Term{Year: year, Season: Spring}, &Term{Year: year, Season: Summer}
	} else if dayOfYear < SummerRange.Start {
		// Spring over, Summer not yet begun
		return nil, &Term{Year: year, Season: Summer}
	} else if (dayOfYear >= SummerRange.Start) && (dayOfYear < SummerRange.End) {
		// Summer
		return &Term{Year: year, Season: Summer}, &Term{Year: year, Season: Fall}
	} else if dayOfYear < FallRange.Start {
		// Summer over, Fall not yet begun
		return nil, &Term{Year: year + 1, Season: Fall}
	} else if (dayOfYear >= FallRange.Start) && (dayOfYear < FallRange.End) {
		// Fall
		return &Term{Year: year + 1, Season: Fall}, nil
	}

	panic(fmt.Sprintf("Impossible Code Reached (dayOfYear: %d)", dayOfYear))
}

// ParseTerm converts a Banner term code to a Term struct
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

// TermToBannerTerm converts a Term struct to a Banner term code
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

// Default chooses the default term, which is the current term if it exists, otherwise the next term.
func Default(t time.Time) Term {
	currentTerm, nextTerm := GetCurrentTerm(t)
	if currentTerm == nil {
		return *nextTerm
	}
	return *currentTerm
}
