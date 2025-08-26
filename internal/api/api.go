package api

import (
	"banner/internal"
	"banner/internal/config"
	"banner/internal/models"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
	"strconv"
	"strings"

	"time"

	"github.com/redis/go-redis/v9"
	"github.com/rs/zerolog/log"
	"github.com/samber/lo"
	"resty.dev/v3"
)

// API provides a client for interacting with the Banner API.
type API struct {
	config *config.Config
}

// New creates a new API client with the given configuration.
func New(config *config.Config) *API {
	return &API{config: config}
}

var (
	latestSession string
	sessionTime   time.Time
	expiryTime    = 25 * time.Minute
)

// SessionMiddleware creates a Resty middleware that resets the session timer on each successful Banner API call.
func SessionMiddleware(_ *resty.Client, r *resty.Response) error {
	// log.Debug().Str("url", r.Request.RawRequest.URL.Path).Msg("Session middleware")

	// Reset session timer on successful requests to Banner API endpoints
	if r.IsSuccess() && strings.HasPrefix(r.Request.RawRequest.URL.Path, "StudentRegistrationSsb/ssb/classSearch/") {
		// Only reset the session time if the session is still valid
		if time.Since(sessionTime) <= expiryTime {
			sessionTime = time.Now()
		}
	}
	return nil
}

// GenerateSession generates a new session ID for use with the Banner API.
// This function should not be used directly; use EnsureSession instead.
func GenerateSession() string {
	return internal.RandomString(5) + internal.Nonce()
}

// DefaultTerm returns the default term, which is the current term if it exists, otherwise the next term.
func (a *API) DefaultTerm(t time.Time) config.Term {
	currentTerm, nextTerm := config.GetCurrentTerm(*a.config.SeasonRanges, t)
	if currentTerm == nil {
		return *nextTerm
	}
	return *currentTerm
}

var terms []BannerTerm
var lastTermUpdate time.Time

// TryReloadTerms attempts to reload the terms if they are not loaded or if the last update was more than 24 hours ago.
func (a *API) TryReloadTerms() error {
	if len(terms) > 0 && time.Since(lastTermUpdate) < 24*time.Hour {
		return nil
	}

	// Load the terms
	var err error
	terms, err = a.GetTerms("", 1, 100)
	if err != nil {
		return fmt.Errorf("failed to load terms: %w", err)
	}

	lastTermUpdate = time.Now()
	return nil
}

// IsTermArchived checks if the given term is archived (view only).
//
// TODO: Add error handling for when a term does not exist.
func (a *API) IsTermArchived(term string) bool {
	// Ensure the terms are loaded
	err := a.TryReloadTerms()
	if err != nil {
		log.Err(err).Stack().Msg("Failed to reload terms")
		return true
	}

	// Check if the term is in the list of terms
	bannerTerm, exists := lo.Find(terms, func(t BannerTerm) bool {
		return t.Code == term
	})

	if !exists {
		log.Warn().Str("term", term).Msg("Term does not exist")
		return true
	}

	return bannerTerm.Archived()
}

// EnsureSession ensures that a valid session is available, creating one if necessary.
func (a *API) EnsureSession() string {
	if latestSession == "" || time.Since(sessionTime) >= expiryTime {
		latestSession = GenerateSession()
		sessionTime = time.Now()
	}
	return latestSession
}

// Pair represents a key-value pair from the Banner API.
type Pair struct {
	Code        string `json:"code"`
	Description string `json:"description"`
}

// BannerTerm represents a term in the Banner system.
type BannerTerm Pair

// Instructor represents an instructor in the Banner system.
type Instructor Pair

// Archived returns true if the term is in an archival (view-only) state.
func (term BannerTerm) Archived() bool {
	return strings.Contains(term.Description, "View Only")
}

// GetTerms retrieves a list of terms from the Banner API.
// The page number must be at least 1.
func (a *API) GetTerms(search string, page int, maxResults int) ([]BannerTerm, error) {
	// Ensure offset is valid
	if page <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := a.config.Client.NewRequest().
		SetQueryParam("searchTerm", search).
		SetQueryParam("offset", strconv.Itoa(page)).
		SetQueryParam("max", strconv.Itoa(maxResults)).
		SetQueryParam("_", internal.Nonce()).
		SetExpectResponseContentType("application/json").
		SetResult(&[]BannerTerm{})

	res, err := req.Get("/classSearch/getTerms")
	if err != nil {
		return nil, fmt.Errorf("failed to get terms: %w", err)
	}

	terms, ok := res.Result().(*[]BannerTerm)
	if !ok {
		return nil, fmt.Errorf("terms parsing failed to cast: %v", res.Result())
	}

	return *terms, nil
}

// SelectTerm selects a term in the Banner system for the given session.
// This is required before other API calls can be made.
func (a *API) SelectTerm(term string, sessionID string) error {
	form := url.Values{
		"term":            {term},
		"studyPath":       {""},
		"studyPathText":   {""},
		"startDatepicker": {""},
		"endDatepicker":   {""},
		"uniqueSessionId": {sessionID},
	}

	type RedirectResponse struct {
		FwdURL string `json:"fwdUrl"`
	}

	req := a.config.Client.NewRequest().
		SetResult(&RedirectResponse{}).
		SetQueryParam("mode", "search").
		SetBody(form.Encode()).
		SetExpectResponseContentType("application/json").
		SetHeader("Content-Type", "application/x-www-form-urlencoded")

	res, err := req.Post("/term/search")
	if err != nil {
		return fmt.Errorf("failed to select term: %w", err)
	}

	redirectResponse := res.Result().(*RedirectResponse)

	// TODO: Mild validation to ensure the redirect is appropriate

	// Make a GET request to the fwdUrl
	req = a.config.Client.NewRequest()
	res, err = req.Get(redirectResponse.FwdURL)

	// Assert that the response is OK (200)
	if res.StatusCode() != 200 {
		return fmt.Errorf("redirect response was not OK: %d", res.StatusCode())
	}

	return nil
}

// GetPartOfTerms retrieves a list of parts of a term from the Banner API.
// The page number must be at least 1.
func (a *API) GetPartOfTerms(search string, term int, offset int, maxResults int) ([]BannerTerm, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := a.config.Client.NewRequest().
		SetQueryParam("searchTerm", search).
		SetQueryParam("term", strconv.Itoa(term)).
		SetQueryParam("offset", strconv.Itoa(offset)).
		SetQueryParam("max", strconv.Itoa(maxResults)).
		SetQueryParam("uniqueSessionId", a.EnsureSession()).
		SetQueryParam("_", internal.Nonce()).
		SetExpectResponseContentType("application/json").
		SetResult(&[]BannerTerm{})

	res, err := req.Get("/classSearch/get_partOfTerm")
	if err != nil {
		return nil, fmt.Errorf("failed to get part of terms: %w", err)
	}

	terms, ok := res.Result().(*[]BannerTerm)
	if !ok {
		return nil, fmt.Errorf("term parsing failed to cast: %v", res.Result())
	}

	return *terms, nil
}

// GetInstructors retrieves a list of instructors from the Banner API.
func (a *API) GetInstructors(search string, term string, offset int, maxResults int) ([]Instructor, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := a.config.Client.NewRequest().
		SetQueryParam("searchTerm", search).
		SetQueryParam("term", term).
		SetQueryParam("offset", strconv.Itoa(offset)).
		SetQueryParam("max", strconv.Itoa(maxResults)).
		SetQueryParam("uniqueSessionId", a.EnsureSession()).
		SetQueryParam("_", internal.Nonce()).
		SetExpectResponseContentType("application/json").
		SetResult(&[]Instructor{})

	res, err := req.Get("/classSearch/get_instructor")
	if err != nil {
		return nil, fmt.Errorf("failed to get instructors: %w", err)
	}

	instructors, ok := res.Result().(*[]Instructor)
	if !ok {
		return nil, fmt.Errorf("instructor parsing failed to cast: %v", res.Result())
	}

	return *instructors, nil
}

// ClassDetails represents the detailed information for a class.
//
// TODO: Implement this struct and the associated GetCourseDetails function.
type ClassDetails struct {
}

// GetCourseDetails retrieves the details for a specific course.
func (a *API) GetCourseDetails(term int, crn int) (*ClassDetails, error) {
	body, err := json.Marshal(map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
		"first":                 "first", // TODO: What is this?
	})
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Failed to marshal body")
	}

	req := a.config.Client.NewRequest().
		SetBody(body).
		SetExpectResponseContentType("application/json").
		SetResult(&ClassDetails{})

	res, err := req.Get("/searchResults/getClassDetails")
	if err != nil {
		return nil, fmt.Errorf("failed to get course details: %w", err)
	}

	details, ok := res.Result().(*ClassDetails)
	if !ok {
		return nil, fmt.Errorf("course details parsing failed to cast: %v", res.Result())
	}

	return details, nil
}

// Search performs a search for courses with the given query and returns the results.
func (a *API) Search(term string, query *Query, sort string, sortDescending bool) (*models.SearchResult, error) {
	a.ResetDataForm()

	params := query.Paramify()

	params["txt_term"] = term
	params["uniqueSessionId"] = a.EnsureSession()
	params["sortColumn"] = sort
	params["sortDirection"] = "asc"

	// These dates are not available for usage anywhere in the UI, but are included in every query
	params["startDatepicker"] = ""
	params["endDatepicker"] = ""

	req := a.config.Client.NewRequest().
		SetQueryParams(params).
		SetExpectResponseContentType("application/json").
		SetResult(&models.SearchResult{})

	res, err := req.Get("/searchResults/searchResults")
	if err != nil {
		return nil, fmt.Errorf("failed to search: %w", err)
	}

	searchResult, ok := res.Result().(*models.SearchResult)
	if !ok {
		return nil, fmt.Errorf("search result parsing failed to cast: %v", res.Result())
	}

	return searchResult, nil
}

// GetSubjects retrieves a list of subjects from the Banner API.
// The page number must be at least 1.
func (a *API) GetSubjects(search string, term string, offset int, maxResults int) ([]Pair, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := a.config.Client.NewRequest().
		SetQueryParam("searchTerm", search).
		SetQueryParam("term", term).
		SetQueryParam("offset", strconv.Itoa(offset)).
		SetQueryParam("max", strconv.Itoa(maxResults)).
		SetQueryParam("uniqueSessionId", a.EnsureSession()).
		SetQueryParam("_", internal.Nonce()).
		SetExpectResponseContentType("application/json").
		SetResult(&[]Pair{})

	res, err := req.Get("/classSearch/get_subject")
	if err != nil {
		return nil, fmt.Errorf("failed to get subjects: %w", err)
	}

	subjects, ok := res.Result().(*[]Pair)
	if !ok {
		return nil, fmt.Errorf("subjects parsing failed to cast: %v", res.Result())
	}

	return *subjects, nil
}

// GetCampuses retrieves a list of campuses from the Banner API.
// The page number must be at least 1.
func (a *API) GetCampuses(search string, term int, offset int, maxResults int) ([]Pair, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := a.config.Client.NewRequest().
		SetQueryParam("searchTerm", search).
		SetQueryParam("term", strconv.Itoa(term)).
		SetQueryParam("offset", strconv.Itoa(offset)).
		SetQueryParam("max", strconv.Itoa(maxResults)).
		SetQueryParam("uniqueSessionId", a.EnsureSession()).
		SetQueryParam("_", internal.Nonce()).
		SetExpectResponseContentType("application/json").
		SetResult(&[]Pair{})

	res, err := req.Get("/classSearch/get_campus")
	if err != nil {
		return nil, fmt.Errorf("failed to get campuses: %w", err)
	}

	campuses, ok := res.Result().(*[]Pair)
	if !ok {
		return nil, fmt.Errorf("campuses parsing failed to cast: %v", res.Result())
	}

	return *campuses, nil
}

// GetInstructionalMethods retrieves a list of instructional methods from the Banner API.
// The page number must be at least 1.
func (a *API) GetInstructionalMethods(search string, term string, offset int, maxResults int) ([]Pair, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := a.config.Client.NewRequest().
		SetQueryParam("searchTerm", search).
		SetQueryParam("term", term).
		SetQueryParam("offset", strconv.Itoa(offset)).
		SetQueryParam("max", strconv.Itoa(maxResults)).
		SetQueryParam("uniqueSessionId", a.EnsureSession()).
		SetQueryParam("_", internal.Nonce()).
		SetExpectResponseContentType("application/json").
		SetResult(&[]Pair{})

	res, err := req.Get("/classSearch/get_instructionalMethod")
	if err != nil {
		return nil, fmt.Errorf("failed to get instructional methods: %w", err)
	}

	methods, ok := res.Result().(*[]Pair)
	if !ok {
		return nil, fmt.Errorf("instructional methods parsing failed to cast: %v", res.Result())
	}
	return *methods, nil
}

// GetCourseMeetingTime retrieves the meeting time information for a course.
func (a *API) GetCourseMeetingTime(term int, crn int) ([]models.MeetingTimeResponse, error) {
	type responseWrapper struct {
		Fmt []models.MeetingTimeResponse `json:"fmt"`
	}

	req := a.config.Client.NewRequest().
		SetQueryParam("term", strconv.Itoa(term)).
		SetQueryParam("courseReferenceNumber", strconv.Itoa(crn)).
		SetExpectResponseContentType("application/json").
		SetResult(&responseWrapper{})

	res, err := req.Get("/searchResults/getFacultyMeetingTimes")
	if err != nil {
		return nil, fmt.Errorf("failed to get meeting time: %w", err)
	}

	result, ok := res.Result().(*responseWrapper)
	if !ok {
		return nil, fmt.Errorf("meeting times parsing failed to cast: %v", res.Result())
	}

	return result.Fmt, nil
}

// ResetDataForm resets the search form in the Banner system.
// This must be called before a new search can be performed.
func (a *API) ResetDataForm() {
	req := a.config.Client.NewRequest()

	_, err := req.Post("/classSearch/resetDataForm")
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Failed to reset data form")
	}
}

// GetCourse retrieves course information from the Redis cache.
func (a *API) GetCourse(crn string) (*models.Course, error) {
	// Create a timeout context for Redis operations
	ctx, cancel := context.WithTimeout(a.config.Ctx, 5*time.Second)
	defer cancel()

	// Retrieve raw data
	result, err := a.config.KV.Get(ctx, fmt.Sprintf("class:%s", crn)).Result()
	if err != nil {
		if err == redis.Nil {
			return nil, fmt.Errorf("course not found: %w", err)
		}
		return nil, fmt.Errorf("failed to get course: %w", err)
	}

	// Unmarshal the raw data
	var course models.Course
	err = json.Unmarshal([]byte(result), &course)
	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal course: %w", err)
	}

	return &course, nil
}
