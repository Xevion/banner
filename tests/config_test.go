package config_test

import (
	"banner/internal/config"
	"testing"
	"time"
)

func TestGetCurrentTerm(t *testing.T) {
	// Initialize location for testing
	loc, _ := time.LoadLocation("America/Chicago")
	
	// Use current year to avoid issues with global state
	currentYear := uint16(time.Now().Year())
	ranges := config.GetYearDayRange(loc, currentYear)

	tests := []struct {
		name            string
		date            time.Time
		expectedCurrent *config.Term
		expectedNext    *config.Term
	}{
		{
			name:            "Spring term",
			date:            time.Date(int(currentYear), 3, 15, 12, 0, 0, 0, loc),
			expectedCurrent: &config.Term{Year: currentYear, Season: config.Spring},
			expectedNext:    &config.Term{Year: currentYear, Season: config.Summer},
		},
		{
			name:            "Summer term",
			date:            time.Date(int(currentYear), 6, 15, 12, 0, 0, 0, loc),
			expectedCurrent: &config.Term{Year: currentYear, Season: config.Summer},
			expectedNext:    &config.Term{Year: currentYear, Season: config.Fall},
		},
		{
			name:            "Fall term",
			date:            time.Date(int(currentYear), 9, 15, 12, 0, 0, 0, loc),
			expectedCurrent: &config.Term{Year: currentYear + 1, Season: config.Fall},
			expectedNext:    nil,
		},
		{
			name:            "Between Spring and Summer",
			date:            time.Date(int(currentYear), 5, 20, 12, 0, 0, 0, loc),
			expectedCurrent: nil,
			expectedNext:    &config.Term{Year: currentYear, Season: config.Summer},
		},
		{
			name:            "Between Summer and Fall",
			date:            time.Date(int(currentYear), 8, 16, 12, 0, 0, 0, loc),
			expectedCurrent: nil,
			expectedNext:    &config.Term{Year: currentYear + 1, Season: config.Fall},
		},
		{
			name:            "Between Fall and Spring",
			date:            time.Date(int(currentYear), 12, 15, 12, 0, 0, 0, loc),
			expectedCurrent: nil,
			expectedNext:    &config.Term{Year: currentYear + 1, Season: config.Spring},
		},
		{
			name:            "Early January before Spring",
			date:            time.Date(int(currentYear), 1, 10, 12, 0, 0, 0, loc),
			expectedCurrent: nil,
			expectedNext:    &config.Term{Year: currentYear, Season: config.Spring},
		},
		{
			name:            "Spring start date",
			date:            time.Date(int(currentYear), 1, 14, 0, 0, 0, 0, loc),
			expectedCurrent: &config.Term{Year: currentYear, Season: config.Spring},
			expectedNext:    &config.Term{Year: currentYear, Season: config.Summer},
		},
		{
			name:            "Summer start date",
			date:            time.Date(int(currentYear), 5, 25, 0, 0, 0, 0, loc),
			expectedCurrent: &config.Term{Year: currentYear, Season: config.Summer},
			expectedNext:    &config.Term{Year: currentYear, Season: config.Fall},
		},
		{
			name:            "Fall start date",
			date:            time.Date(int(currentYear), 8, 18, 0, 0, 0, 0, loc),
			expectedCurrent: &config.Term{Year: currentYear + 1, Season: config.Fall},
			expectedNext:    nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			current, next := config.GetCurrentTerm(ranges, tt.date)

			if !termsEqual(current, tt.expectedCurrent) {
				t.Errorf("GetCurrentTerm() current = %v, want %v", current, tt.expectedCurrent)
			}

			if !termsEqual(next, tt.expectedNext) {
				t.Errorf("GetCurrentTerm() next = %v, want %v", next, tt.expectedNext)
			}
		})
	}
}

func TestGetYearDayRange(t *testing.T) {
	loc, _ := time.LoadLocation("America/Chicago")

	ranges := config.GetYearDayRange(loc, 2024)

	// Verify Spring range (Jan 14 to May 1)
	expectedSpringStart := time.Date(2024, 1, 14, 0, 0, 0, 0, loc).YearDay()
	expectedSpringEnd := time.Date(2024, 5, 1, 0, 0, 0, 0, loc).YearDay()

	if ranges.Spring.Start != uint16(expectedSpringStart) {
		t.Errorf("Spring start = %d, want %d", ranges.Spring.Start, expectedSpringStart)
	}
	if ranges.Spring.End != uint16(expectedSpringEnd) {
		t.Errorf("Spring end = %d, want %d", ranges.Spring.End, expectedSpringEnd)
	}

	// Verify Summer range (May 25 to Aug 15)
	expectedSummerStart := time.Date(2024, 5, 25, 0, 0, 0, 0, loc).YearDay()
	expectedSummerEnd := time.Date(2024, 8, 15, 0, 0, 0, 0, loc).YearDay()

	if ranges.Summer.Start != uint16(expectedSummerStart) {
		t.Errorf("Summer start = %d, want %d", ranges.Summer.Start, expectedSummerStart)
	}
	if ranges.Summer.End != uint16(expectedSummerEnd) {
		t.Errorf("Summer end = %d, want %d", ranges.Summer.End, expectedSummerEnd)
	}

	// Verify Fall range (Aug 18 to Dec 10)
	expectedFallStart := time.Date(2024, 8, 18, 0, 0, 0, 0, loc).YearDay()
	expectedFallEnd := time.Date(2024, 12, 10, 0, 0, 0, 0, loc).YearDay()

	if ranges.Fall.Start != uint16(expectedFallStart) {
		t.Errorf("Fall start = %d, want %d", ranges.Fall.Start, expectedFallStart)
	}
	if ranges.Fall.End != uint16(expectedFallEnd) {
		t.Errorf("Fall end = %d, want %d", ranges.Fall.End, expectedFallEnd)
	}
}

func TestParseTerm(t *testing.T) {
	tests := []struct {
		code     string
		expected config.Term
	}{
		{"202410", config.Term{Year: 2024, Season: config.Fall}},
		{"202420", config.Term{Year: 2024, Season: config.Spring}},
		{"202430", config.Term{Year: 2024, Season: config.Summer}},
		{"202510", config.Term{Year: 2025, Season: config.Fall}},
	}

	for _, tt := range tests {
		t.Run(tt.code, func(t *testing.T) {
			result := config.ParseTerm(tt.code)
			if result != tt.expected {
				t.Errorf("ParseTerm(%s) = %v, want %v", tt.code, result, tt.expected)
			}
		})
	}
}

func TestTermToString(t *testing.T) {
	tests := []struct {
		term     config.Term
		expected string
	}{
		{config.Term{Year: 2024, Season: config.Fall}, "202410"},
		{config.Term{Year: 2024, Season: config.Spring}, "202420"},
		{config.Term{Year: 2024, Season: config.Summer}, "202430"},
		{config.Term{Year: 2025, Season: config.Fall}, "202510"},
	}

	for _, tt := range tests {
		t.Run(tt.expected, func(t *testing.T) {
			result := tt.term.ToString()
			if result != tt.expected {
				t.Errorf("Term{Year: %d, Season: %d}.ToString() = %s, want %s",
					tt.term.Year, tt.term.Season, result, tt.expected)
			}
		})
	}
}

func TestDefaultTerm(t *testing.T) {
	loc, _ := time.LoadLocation("America/Chicago")
	ranges := config.GetYearDayRange(loc, 2024)

	tests := []struct {
		name     string
		date     time.Time
		expected config.Term
	}{
		{
			name:     "During Spring term",
			date:     time.Date(2024, 3, 15, 12, 0, 0, 0, loc),
			expected: config.Term{Year: 2024, Season: config.Spring},
		},
		{
			name:     "Between terms - returns next term",
			date:     time.Date(2024, 5, 20, 12, 0, 0, 0, loc),
			expected: config.Term{Year: 2024, Season: config.Summer},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			currentTerm, nextTerm := config.GetCurrentTerm(ranges, tt.date)
			var result config.Term
			if currentTerm == nil {
				result = *nextTerm
			} else {
				result = *currentTerm
			}
			
			if result != tt.expected {
				t.Errorf("DefaultTerm() = %v, want %v", result, tt.expected)
			}
		})
	}
}

// Helper function to compare terms, handling nil cases
func termsEqual(a, b *config.Term) bool {
	if a == nil && b == nil {
		return true
	}
	if a == nil || b == nil {
		return false
	}
	return *a == *b
}
