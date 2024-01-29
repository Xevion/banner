package main

import (
	"fmt"
	"time"

	"github.com/samber/lo"
)

var (
	// PriorityMajors is a list of majors that are considered to be high priority for scraping. This list is used to determine which majors to scrape first/most often.
	PriorityMajors = []string{"CS", "CPE", "MAT", "EE", "IS"}
	// AncillaryMajors is a list of majors that are considered to be low priority for scraping. This list will not contain any majors that are in PriorityMajors.
	AncillaryMajors = []string{}
)

// Scrape is the general scraping invocation (best called as a goroutine) that should be called regularly to initiate scraping of the Banner system.
func Scrape() error {
	// Populate AllMajors if it is empty
	if len(AncillaryMajors) == 0 {
		subjects, err := GetSubjects("", Default(time.Now()).ToString(), 1, 99)
		if err != nil {
			return fmt.Errorf("failed to get subjects: %w", err)
		}

		// Ensure subjects were found
		if len(subjects) == 0 {
			return fmt.Errorf("no subjects found")
		}

		// Extract major code name
		for _, subject := range subjects {
			// Add to AncillaryMajors if not in PriorityMajors
			if !lo.Contains(PriorityMajors, subject.Code) {
				AncillaryMajors = append(AncillaryMajors, subject.Code)
			}
		}
	}

	for _, subject := range PriorityMajors {
		err := ScrapeMajor(subject)
		if err != nil {
			return fmt.Errorf("failed to scrape priority major %s: %w", subject, err)
		}
	}

	for _, subject := range AncillaryMajors {
		err := ScrapeMajor(subject)
		if err != nil {
			return fmt.Errorf("failed to scrape ancillary major %s: %w", subject, err)
		}
	}

	return nil
}
