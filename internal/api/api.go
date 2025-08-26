package api

import (
	"banner/internal/config"
	"banner/internal/models"
	"banner/internal/utils"
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"

	"time"

	"github.com/redis/go-redis/v9"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/samber/lo"
)

type API struct {
	config *config.Config
}

func New(config *config.Config) *API {
	return &API{config: config}
}

var (
	latestSession string
	sessionTime   time.Time
	expiryTime    = 25 * time.Minute
)

// ResetSessionTimer resets the session timer to the current time.
// This is only used by the DoRequest handler when Banner API calls are detected, which would reset the session timer.
func ResetSessionTimer() {
	// Only reset the session time if the session is still valid
	if time.Since(sessionTime) <= expiryTime {
		sessionTime = time.Now()
	}
}

// GenerateSession generates a new session ID (nonce) for use with the Banner API.
// Don't use this function directly, use GetSession instead.
func GenerateSession() string {
	return utils.RandomString(5) + utils.Nonce()
}

// DoRequest performs & logs the request, logging and returning the response
func (a *API) DoRequest(req *http.Request) (*http.Response, error) {
	headerSize := 0
	for key, values := range req.Header {
		for _, value := range values {
			headerSize += len(key)
			headerSize += len(value)
		}
	}

	bodySize := int64(0)
	if req.Body != nil {
		bodySize, _ = io.Copy(io.Discard, req.Body)
	}

	size := zerolog.Dict().Int64("body", bodySize).Int("header", headerSize).Int("url", len(req.URL.String()))

	log.Debug().
		Dict("size", size).
		Str("method", strings.TrimRight(req.Method, " ")).
		Str("url", req.URL.String()).
		Str("query", req.URL.RawQuery).
		Str("content-type", req.Header.Get("Content-Type")).
		Msg("Request")

	res, err := a.config.Client.Do(req)

	if err != nil {
		log.Err(err).Stack().Str("method", req.Method).Msg("Request Failed")
	} else {
		contentLengthHeader := res.Header.Get("Content-Length")
		contentLength := int64(-1)

		// If this request was a Banner API request, reset the session timer
		if strings.HasPrefix(req.URL.Path, "StudentRegistrationSsb/ssb/classSearch/") {
			ResetSessionTimer()
		}

		// Get the content length
		if contentLengthHeader != "" {
			contentLength, err = strconv.ParseInt(contentLengthHeader, 10, 64)
			if err != nil {
				contentLength = -1
			}
		}

		log.Debug().Int("status", res.StatusCode).Int64("content-length", contentLength).Strs("content-type", res.Header["Content-Type"]).Msg("Response")
	}
	return res, err
}

var terms []BannerTerm
var lastTermUpdate time.Time

// TryReloadTerms attempts to reload the terms if they are not loaded or the last update was more than 24 hours ago
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

// IsTermArchived checks if the given term is archived
// TODO: Add error, switch missing term logic to error
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

type Pair struct {
	Code        string `json:"code"`
	Description string `json:"description"`
}

type BannerTerm Pair
type Instructor Pair

// Archived returns true if the term is in it's archival state (view only)
func (term BannerTerm) Archived() bool {
	return strings.Contains(term.Description, "View Only")
}

// GetTerms retrieves and parses the term information for a given search term.
// Page number must be at least 1.
func (a *API) GetTerms(search string, page int, maxResults int) ([]BannerTerm, error) {
	// Ensure offset is valid
	if page <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := utils.BuildRequest(a.config, "GET", "/classSearch/getTerms", map[string]string{
		"searchTerm": search,
		// Page vs Offset is not a mistake here, the API uses "offset" as the page number
		"offset": strconv.Itoa(page),
		"max":    strconv.Itoa(maxResults),
		"_":      utils.Nonce(),
	})

	if page <= 0 {
		return nil, errors.New("Offset must be greater than 0")
	}

	res, err := a.DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get terms: %w", err)
	}

	// Assert that the response is JSON
	if contentType := res.Header.Get("Content-Type"); !strings.Contains(contentType, models.JsonContentType) {
		return nil, &utils.UnexpectedContentTypeError{
			Expected: models.JsonContentType,
			Actual:   contentType,
		}
	}

	// print the response body
	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	terms := make([]BannerTerm, 0, 10)
	err = json.Unmarshal(body, &terms)

	if err != nil {
		return nil, fmt.Errorf("failed to parse terms: %w", err)
	}

	return terms, nil
}

// SelectTerm selects the given term in the Banner system.
// This function completes the initial term selection process, which is required before any other API calls can be made with the session ID.
func (a *API) SelectTerm(term string, sessionID string) error {
	form := url.Values{
		"term":            {term},
		"studyPath":       {""},
		"studyPathText":   {""},
		"startDatepicker": {""},
		"endDatepicker":   {""},
		"uniqueSessionId": {sessionID},
	}

	params := map[string]string{
		"mode": "search",
	}

	req := utils.BuildRequestWithBody(a.config, "POST", "/term/search", params, bytes.NewBufferString(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	res, err := a.DoRequest(req)
	if err != nil {
		return fmt.Errorf("failed to select term: %w", err)
	}

	// Assert that the response is JSON
	if !utils.ContentTypeMatch(res, "application/json") {
		return fmt.Errorf("response was not JSON: %s", res.Header.Get("Content-Type"))
	}

	// Acquire fwdUrl
	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return fmt.Errorf("failed to read response body: %w", err)
	}

	var redirectResponse struct {
		FwdURL string `json:"fwdUrl"`
	}
	json.Unmarshal(body, &redirectResponse)

	// Make a GET request to the fwdUrl
	req = utils.BuildRequest(a.config, "GET", redirectResponse.FwdURL, nil)
	res, err = a.DoRequest(req)
	if err != nil {
		return fmt.Errorf("failed to follow redirect: %w", err)
	}

	// Assert that the response is OK (200)
	if res.StatusCode != 200 {
		return fmt.Errorf("redirect response was not 200: %d", res.StatusCode)
	}

	return nil
}

// GetPartOfTerms retrieves and parses the part of term information for a given term.
// Ensure that the offset is greater than 0.
func (a *API) GetPartOfTerms(search string, term int, offset int, maxResults int) ([]BannerTerm, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := utils.BuildRequest(a.config, "GET", "/classSearch/get_partOfTerm", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(maxResults),
		"uniqueSessionId": a.EnsureSession(),
		"_":               utils.Nonce(),
	})

	res, err := a.DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get part of terms: %w", err)
	}

	// Assert that the response is JSON
	if !utils.ContentTypeMatch(res, "application/json") {
		log.Panic().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	terms := make([]BannerTerm, 0, 10)
	err = json.Unmarshal(body, &terms)
	if err != nil {
		return nil, fmt.Errorf("failed to parse part of terms: %w", err)
	}

	return terms, nil
}

// GetInstructors retrieves and parses the instructor information for a given search term.
// In my opinion, it is unclear what providing the term does, as the results should be the same regardless of the term.
// This function is included for completeness, but probably isn't useful.
// Ensure that the offset is greater than 0.
func (a *API) GetInstructors(search string, term string, offset int, maxResults int) ([]Instructor, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := utils.BuildRequest(a.config, "GET", "/classSearch/get_instructor", map[string]string{
		"searchTerm":      search,
		"term":            term,
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(maxResults),
		"uniqueSessionId": a.EnsureSession(),
		"_":               utils.Nonce(),
	})

	res, err := a.DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get instructors: %w", err)
	}

	// Assert that the response is JSON
	if !utils.ContentTypeMatch(res, "application/json") {
		log.Fatal().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	instructors := make([]Instructor, 0, 10)
	err = json.Unmarshal(body, &instructors)
	if err != nil {
		return nil, fmt.Errorf("failed to parse instructors: %w", err)
	}

	return instructors, nil
}

// ClassDetails represents the details of a course.
// TODO: Finish this struct & function
type ClassDetails struct {
}

func (a *API) GetCourseDetails(term int, crn int) *ClassDetails {
	body, err := json.Marshal(map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
		"first":                 "first", // TODO: What is this?
	})
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Failed to marshal body")
	}
	req := utils.BuildRequestWithBody(a.config, "GET", "/searchResults/getClassDetails", nil, bytes.NewBuffer(body))

	res, err := a.DoRequest(req)
	if err != nil {
		return nil
	}

	// Assert that the response is JSON
	if !utils.ContentTypeMatch(res, "application/json") {
		log.Fatal().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	return &ClassDetails{}
}

// Search invokes a search on the Banner system with the given query and returns the results.
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

	req := utils.BuildRequest(a.config, "GET", "/searchResults/searchResults", params)

	res, err := a.DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to search: %w", err)
	}

	if res.StatusCode != 200 {
		return nil, fmt.Errorf("search failed with status code: %d", res.StatusCode)
	}

	// Assert that the response is JSON
	if !utils.ContentTypeMatch(res, "application/json") {
		// for server 500 errors, parse for the error with '#dialog-message > div.message'
		log.Error().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	var result models.SearchResult
	err = json.Unmarshal(body, &result)

	if err != nil {
		return nil, fmt.Errorf("failed to parse search results: %w", err)
	}

	return &result, nil
}

// GetSubjects retrieves and parses the subject information for a given search term.
// The results of this response shouldn't change much, but technically could as new majors are developed, or old ones are removed.
// Ensure that the offset is greater than 0.
func (a *API) GetSubjects(search string, term string, offset int, maxResults int) ([]Pair, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := utils.BuildRequest(a.config, "GET", "/classSearch/get_subject", map[string]string{
		"searchTerm":      search,
		"term":            term,
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(maxResults),
		"uniqueSessionId": a.EnsureSession(),
		"_":               utils.Nonce(),
	})

	res, err := a.DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get subjects: %w", err)
	}

	// Assert that the response is JSON
	if !utils.ContentTypeMatch(res, "application/json") {
		log.Fatal().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	subjects := make([]Pair, 0, 10)
	err = json.Unmarshal(body, &subjects)
	if err != nil {
		return nil, fmt.Errorf("failed to parse subjects: %w", err)
	}

	return subjects, nil
}

// GetCampuses retrieves and parses the campus information for a given search term.
// In my opinion, it is unclear what providing the term does, as the results should be the same regardless of the term.
// This function is included for completeness, but probably isn't useful.
// Ensure that the offset is greater than 0.
func (a *API) GetCampuses(search string, term int, offset int, maxResults int) ([]Pair, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := utils.BuildRequest(a.config, "GET", "/classSearch/get_campus", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(maxResults),
		"uniqueSessionId": a.EnsureSession(),
		"_":               utils.Nonce(),
	})

	res, err := a.DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get campuses: %w", err)
	}

	// Assert that the response is JSON
	if !utils.ContentTypeMatch(res, "application/json") {
		log.Fatal().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	campuses := make([]Pair, 0, 10)
	err = json.Unmarshal(body, &campuses)
	if err != nil {
		return nil, fmt.Errorf("failed to parse campuses: %w", err)
	}

	return campuses, nil
}

// GetInstructionalMethods retrieves and parses the instructional method information for a given search term.
// In my opinion, it is unclear what providing the term does, as the results should be the same regardless of the term.
// This function is included for completeness, but probably isn't useful.
// Ensure that the offset is greater than 0.
func (a *API) GetInstructionalMethods(search string, term string, offset int, maxResults int) ([]Pair, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := utils.BuildRequest(a.config, "GET", "/classSearch/get_instructionalMethod", map[string]string{
		"searchTerm":      search,
		"term":            term,
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(maxResults),
		"uniqueSessionId": a.EnsureSession(),
		"_":               utils.Nonce(),
	})

	res, err := a.DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get instructional methods: %w", err)
	}

	// Assert that the response is JSON
	if !utils.ContentTypeMatch(res, "application/json") {
		log.Fatal().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	defer res.Body.Close()
	body, _ := io.ReadAll(res.Body)

	methods := make([]Pair, 0, 10)
	err = json.Unmarshal(body, &methods)
	if err != nil {
		return nil, fmt.Errorf("failed to parse instructional methods: %w", err)
	}

	return methods, nil
}

// GetCourseMeetingTime retrieves the meeting time information for a course based on the given term and course reference number (CRN).
// It makes an HTTP GET request to the appropriate API endpoint and parses the response to extract the meeting time data.
// The function returns a MeetingTimeResponse struct containing the extracted information.
func (a *API) GetCourseMeetingTime(term int, crn int) ([]models.MeetingTimeResponse, error) {
	req := utils.BuildRequest(a.config, "GET", "/searchResults/getFacultyMeetingTimes", map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
	})

	res, err := a.DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get meeting time: %w", err)
	}

	// Assert that the response is JSON
	if !utils.ContentTypeMatch(res, "application/json") {
		log.Fatal().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	// Read the response body into JSON
	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	// Parse the JSON into a MeetingTimeResponse struct
	var meetingTime struct {
		Inner []models.MeetingTimeResponse `json:"fmt"`
	}
	err = json.Unmarshal(body, &meetingTime)
	if err != nil {
		return nil, fmt.Errorf("failed to parse meeting time: %w", err)
	}

	return meetingTime.Inner, nil
}

// ResetDataForm makes a POST request that needs to be made upon before new search requests can be made.
func (a *API) ResetDataForm() {
	req := utils.BuildRequest(a.config, "POST", "/classSearch/resetDataForm", nil)
	_, err := a.DoRequest(req)
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Failed to reset data form")
	}
}

// GetCourse retrieves the course information.
// This course does not retrieve directly from the API, but rather uses scraped data stored in Redis.
func (a *API) GetCourse(crn string) (*models.Course, error) {
	// Retrieve raw data
	result, err := a.config.KV.Get(a.config.Ctx, fmt.Sprintf("class:%s", crn)).Result()
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
