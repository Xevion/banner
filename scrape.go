package main

import (
	"fmt"
	"math/rand"
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
	offset := 0
	totalClassCount := 0
	log.Info().Str("subject", subject).Msg("Scraping Major")

	for {
		query := NewQuery().Offset(offset).MaxResults(MaxPageSize).Subject(subject)
		result, err := Search(query, "subjectDescription", false)
		if err != nil {
			return fmt.Errorf("search failed: %w (%s)", err, query.String())
		}

		if !result.Success {
			return fmt.Errorf("result marked unsuccessful when searching for classes (%s)", query.String())
		}

		classCount := len(result.Data)
		totalClassCount += classCount
		log.Debug().Str("subject", subject).Int("count", classCount).Int("offset", offset).Msg("Placing classes in Redis")

		// Process each class and store it in Redis
		for _, class := range result.Data {
			// TODO: Move this into a separate function to allow future comparison/SQLite intake
			// Store class in Redis
			err := kv.Set(ctx, fmt.Sprintf("class:%s", class.CourseReferenceNumber), class, 0).Err()
			if err != nil {
				log.Error().Err(err).Msg("failed to store class in Redis")
			}
		}

		// Increment and continue if the results are full
		if classCount >= MaxPageSize {
			// This is unlikely to happen, but log it just in case
			if classCount > MaxPageSize {
				log.Warn().Int("page", offset).Int("count", classCount).Msg("Results exceed MaxPageSize")
			}

			offset += MaxPageSize

			// TODO: Replace sleep with smarter rate limiting
			log.Debug().Str("subject", subject).Int("nextOffset", offset).Msg("Sleeping before next page")
			time.Sleep(time.Second * 3)
			continue
		} else {
			// Log the number of classes scraped
			log.Info().Str("subject", subject).Int("total", totalClassCount).Msg("Major Scraped")
			break
		}
	}

	// Calculate the expiry time for the scrape (1 hour for every 200 classes, random +-15%) with a minimum of 1 hour
	scrapeExpiry := time.Hour * time.Duration(totalClassCount/100)
	partial := scrapeExpiry.Seconds() * (rand.Float64() * 0.15) // Between 0 and 15% of the total
	if rand.Intn(2) == 0 {                                      // Randomly add or subtract the partial (delta between -15% and 15%)
		scrapeExpiry -= time.Duration(partial) * time.Second
	} else {
		scrapeExpiry += time.Duration(partial) * time.Second
	}

	// Ensure the expiry is at least 1 hour with up to 15 extra minutes
	if scrapeExpiry < time.Hour {
		scrapeExpiry = time.Hour + time.Duration(rand.Intn(60*15))*time.Second
	}

	// If the subject is a priority, then the expiry is halved
	if lo.Contains(PriorityMajors, subject) {
		scrapeExpiry /= 3
	}

	// Mark the major as scraped
	term := Default(time.Now()).ToString()
	err := kv.Set(ctx, fmt.Sprintf("scraped:%s:%s", subject, term), "1", scrapeExpiry).Err()
	if err != nil {
		log.Error().Err(err).Msg("failed to mark major as scraped")
	} else {
		log.Debug().Str("subject", subject).Str("expiry", scrapeExpiry.String()).Msg("Marked major as scraped")
	}

	return nil
}
