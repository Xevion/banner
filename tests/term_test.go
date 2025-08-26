package utils_test

import (
	"banner/internal/config"
	"banner/internal/utils"
	"testing"
	"time"
)

func TestGetCurrentTerm(t *testing.T) {
	// Initialize config for testing
	config.CentralTimeLocation, _ = time.LoadLocation("America/Chicago")

	// Use current year to avoid issues with global state
	currentYear := uint16(time.Now().Year())

	tests := []struct {
		name            string
		date            time.Time
		expectedCurrent *utils.Term
		expectedNext    *utils.Term
	}{
		{
			name:            "Spring term",
			date:            time.Date(int(currentYear), 3, 15, 12, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: &utils.Term{Year: currentYear, Season: utils.Spring},
			expectedNext:    &utils.Term{Year: currentYear, Season: utils.Summer},
		},
		{
			name:            "Summer term",
			date:            time.Date(int(currentYear), 6, 15, 12, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: &utils.Term{Year: currentYear, Season: utils.Summer},
			expectedNext:    &utils.Term{Year: currentYear, Season: utils.Fall},
		},
		{
			name:            "Fall term",
			date:            time.Date(int(currentYear), 9, 15, 12, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: &utils.Term{Year: currentYear + 1, Season: utils.Fall},
			expectedNext:    nil,
		},
		{
			name:            "Between Spring and Summer",
			date:            time.Date(int(currentYear), 5, 20, 12, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: nil,
			expectedNext:    &utils.Term{Year: currentYear, Season: utils.Summer},
		},
		{
			name:            "Between Summer and Fall",
			date:            time.Date(int(currentYear), 8, 16, 12, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: nil,
			expectedNext:    &utils.Term{Year: currentYear + 1, Season: utils.Fall},
		},
		{
			name:            "Between Fall and Spring",
			date:            time.Date(int(currentYear), 12, 15, 12, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: nil,
			expectedNext:    &utils.Term{Year: currentYear + 1, Season: utils.Spring},
		},
		{
			name:            "Early January before Spring",
			date:            time.Date(int(currentYear), 1, 10, 12, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: nil,
			expectedNext:    &utils.Term{Year: currentYear, Season: utils.Spring},
		},
		{
			name:            "Spring start date",
			date:            time.Date(int(currentYear), 1, 14, 0, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: &utils.Term{Year: currentYear, Season: utils.Spring},
			expectedNext:    &utils.Term{Year: currentYear, Season: utils.Summer},
		},
		{
			name:            "Summer start date",
			date:            time.Date(int(currentYear), 5, 25, 0, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: &utils.Term{Year: currentYear, Season: utils.Summer},
			expectedNext:    &utils.Term{Year: currentYear, Season: utils.Fall},
		},
		{
			name:            "Fall start date",
			date:            time.Date(int(currentYear), 8, 18, 0, 0, 0, 0, config.CentralTimeLocation),
			expectedCurrent: &utils.Term{Year: currentYear + 1, Season: utils.Fall},
			expectedNext:    nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			current, next := utils.GetCurrentTerm(tt.date)

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
	config.CentralTimeLocation, _ = time.LoadLocation("America/Chicago")

	spring, summer, fall := utils.GetYearDayRange(2024)

	// Verify Spring range (Jan 14 to May 1)
	expectedSpringStart := time.Date(2024, 1, 14, 0, 0, 0, 0, config.CentralTimeLocation).YearDay()
	expectedSpringEnd := time.Date(2024, 5, 1, 0, 0, 0, 0, config.CentralTimeLocation).YearDay()

	if spring.Start != uint16(expectedSpringStart) {
		t.Errorf("Spring start = %d, want %d", spring.Start, expectedSpringStart)
	}
	if spring.End != uint16(expectedSpringEnd) {
		t.Errorf("Spring end = %d, want %d", spring.End, expectedSpringEnd)
	}

	// Verify Summer range (May 25 to Aug 15)
	expectedSummerStart := time.Date(2024, 5, 25, 0, 0, 0, 0, config.CentralTimeLocation).YearDay()
	expectedSummerEnd := time.Date(2024, 8, 15, 0, 0, 0, 0, config.CentralTimeLocation).YearDay()

	if summer.Start != uint16(expectedSummerStart) {
		t.Errorf("Summer start = %d, want %d", summer.Start, expectedSummerStart)
	}
	if summer.End != uint16(expectedSummerEnd) {
		t.Errorf("Summer end = %d, want %d", summer.End, expectedSummerEnd)
	}

	// Verify Fall range (Aug 18 to Dec 10)
	expectedFallStart := time.Date(2024, 8, 18, 0, 0, 0, 0, config.CentralTimeLocation).YearDay()
	expectedFallEnd := time.Date(2024, 12, 10, 0, 0, 0, 0, config.CentralTimeLocation).YearDay()

	if fall.Start != uint16(expectedFallStart) {
		t.Errorf("Fall start = %d, want %d", fall.Start, expectedFallStart)
	}
	if fall.End != uint16(expectedFallEnd) {
		t.Errorf("Fall end = %d, want %d", fall.End, expectedFallEnd)
	}
}

func TestParseTerm(t *testing.T) {
	tests := []struct {
		code     string
		expected utils.Term
	}{
		{"202410", utils.Term{Year: 2024, Season: utils.Fall}},
		{"202420", utils.Term{Year: 2024, Season: utils.Spring}},
		{"202430", utils.Term{Year: 2024, Season: utils.Summer}},
		{"202510", utils.Term{Year: 2025, Season: utils.Fall}},
	}

	for _, tt := range tests {
		t.Run(tt.code, func(t *testing.T) {
			result := utils.ParseTerm(tt.code)
			if result != tt.expected {
				t.Errorf("ParseTerm(%s) = %v, want %v", tt.code, result, tt.expected)
			}
		})
	}
}

func TestTermToString(t *testing.T) {
	tests := []struct {
		term     utils.Term
		expected string
	}{
		{utils.Term{Year: 2024, Season: utils.Fall}, "202410"},
		{utils.Term{Year: 2024, Season: utils.Spring}, "202420"},
		{utils.Term{Year: 2024, Season: utils.Summer}, "202430"},
		{utils.Term{Year: 2025, Season: utils.Fall}, "202510"},
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

func TestDefault(t *testing.T) {
	config.CentralTimeLocation, _ = time.LoadLocation("America/Chicago")

	tests := []struct {
		name     string
		date     time.Time
		expected utils.Term
	}{
		{
			name:     "During Spring term",
			date:     time.Date(2024, 3, 15, 12, 0, 0, 0, config.CentralTimeLocation),
			expected: utils.Term{Year: 2024, Season: utils.Spring},
		},
		{
			name:     "Between terms - returns next term",
			date:     time.Date(2024, 5, 20, 12, 0, 0, 0, config.CentralTimeLocation),
			expected: utils.Term{Year: 2024, Season: utils.Summer},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := utils.Default(tt.date)
			if result != tt.expected {
				t.Errorf("Default() = %v, want %v", result, tt.expected)
			}
		})
	}
}

// Helper function to compare terms, handling nil cases
func termsEqual(a, b *utils.Term) bool {
	if a == nil && b == nil {
		return true
	}
	if a == nil || b == nil {
		return false
	}
	return *a == *b
}
