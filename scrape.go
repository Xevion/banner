package main

import (
	"fmt"
	"math/rand"
	"time"

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
	AncillaryMajors []string
	// AllMajors is a list of all majors that are available in the Banner system.
	AllMajors []string
)

// Scrape is the general scraping invocation (best called within/as a goroutine) that should be called regularly to initiate scraping of the Banner system.
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

		AllMajors = lo.Flatten([][]string{PriorityMajors, AncillaryMajors})
	}

	expiredSubjects, err := GetExpiredSubjects()
	if err != nil {
		return fmt.Errorf("failed to get scrapable majors: %w", err)
	}

	log.Info().Strs("majors", expiredSubjects).Msg("Scraping majors")
	for _, subject := range expiredSubjects {
		err := ScrapeMajor(subject)
		if err != nil {
			return fmt.Errorf("failed to scrape major %s: %w", subject, err)
		}
	}

	return nil
}

// GetExpiredSubjects returns a list of subjects that are expired and should be scraped.
func GetExpiredSubjects() ([]string, error) {
	term := Default(time.Now()).ToString()
	subjects := make([]string, 0)

	// Get all subjects
	values, err := kv.MGet(ctx, lo.Map(AllMajors, func(major string, _ int) string {
		return fmt.Sprintf("scraped:%s:%s", major, term)
	})...).Result()
	if err != nil {
		return nil, fmt.Errorf("failed to get all subjects: %w", err)
	}

	// Extract expired subjects
	for i, value := range values {
		subject := AllMajors[i]

		// If the value is nil or "0", then the subject is expired
		if value == nil || value == "0" {
			subjects = append(subjects, subject)
		}
	}

	log.Debug().Strs("majors", subjects).Msg("Expired Subjects")

	return subjects, nil
}

// ScrapeMajor is the scraping invocation for a specific major.
// This function does not check whether scraping is required at this time, it is assumed that the caller has already done so.
func ScrapeMajor(subject string) error {
	offset := 0
	totalClassCount := 0

	for {
		// Build & execute the query
		query := NewQuery().Offset(offset).MaxResults(MaxPageSize * 2).Subject(subject)
		result, err := Search(query, "subjectDescription", false)
		if err != nil {
			return fmt.Errorf("search failed: %w (%s)", err, query.String())
		}

		// Isn't it bullshit that they decided not to leave an actual 'reason' field for the failure?
		if !result.Success {
			return fmt.Errorf("result marked unsuccessful when searching for classes (%s)", query.String())
		}

		classCount := len(result.Data)
		totalClassCount += classCount
		log.Debug().Str("subject", subject).Int("count", classCount).Int("offset", offset).Msg("Placing classes in Redis")

		// Process each class and store it in Redis
		for _, course := range result.Data {
			// Store class in Redis
			err := IntakeCourse(course)
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
			log.Info().Str("subject", subject).Int("total", totalClassCount).Msgf("Subject %s Scraped", subject)
			break
		}
	}

	// Calculate the expiry time for the scrape (1 hour for every 200 classes, random +-15%) with a minimum of 1 hour
	scrapeExpiry := CalculateExpiry(totalClassCount, lo.Contains(PriorityMajors, subject))

	// Mark the major as scraped
	term := Default(time.Now()).ToString()
	err := kv.Set(ctx, fmt.Sprintf("scraped:%s:%s", subject, term), totalClassCount, scrapeExpiry).Err()
	if err != nil {
		log.Error().Err(err).Msg("failed to mark major as scraped")
	}

	return nil
}

// CalculateExpiry calculates the expiry time until the next scrape for a major.
func CalculateExpiry(count int, priority bool) time.Duration {
	scrapeExpiry := time.Hour * time.Duration(count/100)
	partial := scrapeExpiry.Seconds() * (rand.Float64() * 0.15) // Between 0 and 15% of the total

	// Randomly add or subtract the partial (delta between -15% and 15%)
	if rand.Intn(2) == 0 {
		scrapeExpiry -= time.Duration(partial) * time.Second
	} else {
		scrapeExpiry += time.Duration(partial) * time.Second
	}

	// Ensure the expiry is at least 1 hour with up to 15 extra minutes
	if scrapeExpiry < time.Hour {
		scrapeExpiry = time.Hour + time.Duration(rand.Intn(60*15))*time.Second
	}

	// If the subject is a priority, then the expiry is halved
	if priority {
		return scrapeExpiry / 3
	}
	return scrapeExpiry
}

// IntakeCourse stores a course in Redis.
// This function is mostly a stub for now, but will be used to handle change identification, notifications, and SQLite upserts in the future.
func IntakeCourse(course Course) error {
	err := kv.Set(ctx, fmt.Sprintf("class:%s", course.CourseReferenceNumber), course, 0).Err()
	if err != nil {
		return fmt.Errorf("failed to store class in Redis: %w", err)
	}
	return nil
}
