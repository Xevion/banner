// Package api provides the core functionality for interacting with the Banner API.
package api

import (
	"banner/internal"
	"banner/internal/models"
	"context"
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
	// PriorityMajors is a list of majors that are considered to be high priority for scraping.
	// This list is used to determine which majors to scrape first/most often.
	PriorityMajors = []string{"CS", "CPE", "MAT", "EE", "IS"}
	// AncillaryMajors is a list of majors that are considered to be low priority for scraping.
	// This list will not contain any majors that are in PriorityMajors.
	AncillaryMajors []string
	// AllMajors is a list of all majors that are available in the Banner system.
	AllMajors []string
)

// Scrape retrieves all courses from the Banner API and stores them in Redis.
// This is a long-running process that should be run in a goroutine.
//
// TODO: Switch from hardcoded term to dynamic term
func (a *API) Scrape() error {
	// For each subject, retrieve all courses
	// For each course, get the details and store it in redis
	// Make sure to handle pagination
	subjects, err := a.GetSubjects("", "202510", 1, 100)
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

	expiredSubjects, err := a.GetExpiredSubjects()
	if err != nil {
		return fmt.Errorf("failed to get scrapable majors: %w", err)
	}

	log.Info().Strs("majors", expiredSubjects).Msg("Scraping majors")
	for _, subject := range expiredSubjects {
		err := a.ScrapeMajor(subject)
		if err != nil {
			return fmt.Errorf("failed to scrape major %s: %w", subject, err)
		}
	}

	return nil
}

// GetExpiredSubjects returns a list of subjects that have expired and should be scraped again.
// It checks Redis for the "scraped" status of each major for the current term.
func (a *API) GetExpiredSubjects() ([]string, error) {
	term := Default(time.Now()).ToString()
	subjects := make([]string, 0)

	// Create a timeout context for Redis operations
	ctx, cancel := context.WithTimeout(a.config.Ctx, 10*time.Second)
	defer cancel()

	// Get all subjects
	values, err := a.config.KV.MGet(ctx, lo.Map(AllMajors, func(major string, _ int) string {
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

// ScrapeMajor scrapes all courses for a specific major.
// This function does not check whether scraping is required at this time; it is assumed that the caller has already done so.
func (a *API) ScrapeMajor(subject string) error {
	offset := 0
	totalClassCount := 0

	for {
		// Build & execute the query
		query := NewQuery().Offset(offset).MaxResults(MaxPageSize * 2).Subject(subject)
		term := Default(time.Now()).ToString()
		result, err := a.Search(term, query, "subjectDescription", false)
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
			err := a.IntakeCourse(course)
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
		}
		// Log the number of classes scraped
		log.Info().Str("subject", subject).Int("total", totalClassCount).Msgf("Subject %s Scraped", subject)
		break
	}

	term := Default(time.Now()).ToString()

	// Calculate the expiry time for the scrape (1 hour for every 200 classes, random +-15%) with a minimum of 1 hour
	var scrapeExpiry time.Duration
	if totalClassCount == 0 {
		scrapeExpiry = time.Hour * 12
	} else {
		scrapeExpiry = a.CalculateExpiry(term, totalClassCount, lo.Contains(PriorityMajors, subject))
	}

	// Mark the major as scraped
	if totalClassCount == 0 {
		totalClassCount = -1
	}

	// Create a timeout context for Redis operations
	ctx, cancel := context.WithTimeout(a.config.Ctx, 5*time.Second)
	defer cancel()

	err := a.config.KV.Set(ctx, fmt.Sprintf("scraped:%s:%s", subject, term), totalClassCount, scrapeExpiry).Err()
	if err != nil {
		log.Error().Err(err).Msg("failed to mark major as scraped")
	}

	return nil
}

// CalculateExpiry calculates the expiry time until the next scrape for a major.
// The duration is based on the number of courses, whether the major is a priority, and if the term is archived.
func (a *API) CalculateExpiry(term string, count int, priority bool) time.Duration {
	// An hour for every 100 classes
	baseExpiry := time.Hour * time.Duration(count/100)

	// Subjects with less than 50 classes have a reversed expiry (less classes, longer interval)
	// 1 class => 12 hours, 49 classes => 1 hour
	if count < 50 {
		hours := internal.Slope(internal.Point{X: 1, Y: 12}, internal.Point{X: 49, Y: 1}, float64(count)).Y
		baseExpiry = time.Duration(hours * float64(time.Hour))
	}

	// If the subject is a priority, then the expiry is halved without variance
	if priority {
		return baseExpiry / 3
	}

	// If the term is considered "view only" or "archived", then the expiry is multiplied by 5
	var expiry = baseExpiry
	if a.IsTermArchived(term) {
		expiry *= 5
	}

	// Add minor variance to the expiry
	expiryVariance := baseExpiry.Seconds() * (rand.Float64() * 0.15) // Between 0 and 15% of the total
	if rand.Intn(2) == 0 {
		expiry -= time.Duration(expiryVariance) * time.Second
	} else {
		expiry += time.Duration(expiryVariance) * time.Second
	}

	// Ensure the expiry is at least 1 hour with up to 15 extra minutes
	if expiry < time.Hour {
		baseExpiry = time.Hour + time.Duration(rand.Intn(60*15))*time.Second
	}

	return baseExpiry
}

// IntakeCourse stores a course in Redis.
// This function will be used to handle change identification, notifications, and SQLite upserts in the future.
func (a *API) IntakeCourse(course models.Course) error {
	// Create a timeout context for Redis operations
	ctx, cancel := context.WithTimeout(a.config.Ctx, 5*time.Second)
	defer cancel()

	err := a.config.KV.Set(ctx, fmt.Sprintf("class:%s", course.CourseReferenceNumber), course, 0).Err()
	if err != nil {
		return fmt.Errorf("failed to store class in Redis: %w", err)
	}
	return nil
}
