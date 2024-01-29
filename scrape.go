package main

import (
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/rs/zerolog/log"
	"github.com/samber/lo"
)

const (
	MaxPageSize = 500
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
		term := Default(time.Now()).ToString()
		subjects, err := GetSubjects("", term, 1, 99)
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
		if !CanScrape(subject) {
			continue
		}

		err := ScrapeMajor(subject)
		if err != nil {
			return fmt.Errorf("failed to scrape priority major %s: %w", subject, err)
		}
	}

	for _, subject := range AncillaryMajors {
		if !CanScrape(subject) {
			continue
		}

		err := ScrapeMajor(subject)
		if err != nil {
			return fmt.Errorf("failed to scrape ancillary major %s: %w", subject, err)
		}
	}

	return nil
}

// CanScrape returns true if scraping is suggested for a given major at this time.
func CanScrape(subject string) bool {
	term := Default(time.Now()).ToString()
	scraped, err := kv.Get(ctx, fmt.Sprintf("scraped:%s:%s", subject, term)).Result()

	if err != nil {
		// If the key does not exist, then it was never scraped or the scrape needs to be redone (it expired)
		if err == redis.Nil {
			return true
		}

		log.Error().Err(err).Msg("failed to check if scraping is required")
		return false
	}

	return scraped == "1"
}

// ScrapeMajor is the scraping invocation for a specific major.
// This function does not check whether scraping is required at this time, it is assumed that the caller has already done so.
func ScrapeMajor(subject string) error {
	offset := 1
	totalClassCount := 0

	for {
		query := NewQuery().Offset(offset).MaxResults(MaxPageSize)
		result, err := Search(query, "", false)
		if err != nil {
			return fmt.Errorf("failed to search for classes on page %d: %w", offset, err)
		}

		if !result.Success {
			// TODO: Improve error log details
			return fmt.Errorf("search for classes on page %d was not successful", offset)
		}

		// Process each class and store it in Redis
		for _, class := range result.Data {
			// Store class in Redis
			err := kv.Set(ctx, fmt.Sprintf("class:%s", class.CourseReferenceNumber), class, 0).Err()
			if err != nil {
				log.Error().Err(err).Msg("failed to store class in Redis")
			}
		}

		classCount := len(result.Data)
		totalClassCount += classCount

		// Increment and continue if the results are full
		if classCount >= MaxPageSize {
			// This is unlikely to happen, but log it just in case
			if classCount > MaxPageSize {
				log.Warn().Int("page", offset).Int("count", classCount).Msg("Results exceed MaxPageSize")
			}

			offset += MaxPageSize

			// TODO: Replace sleep with smarter rate limiting
			time.Sleep(time.Second * 7)
			continue
		} else {
			// Log the number of classes scraped
			log.Info().Str("subject", subject).Int("count", totalClassCount).Int("offset", offset).Int("finalOffset", offset+classCount).Msg("Scraped classes")
			break
		}
	}

	// Mark the major as scraped
	term := Default(time.Now()).ToString()
	err := kv.Set(ctx, fmt.Sprintf("scraped:%s:%s", subject, term), "1", 0).Err()
	if err != nil {
		log.Error().Err(err).Msg("failed to mark major as scraped")
	}

	return nil
}
