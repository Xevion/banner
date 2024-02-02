package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/url"
	"strconv"
	"strings"

	"github.com/redis/go-redis/v9"
	"github.com/rs/zerolog/log"
)

var sessionID string = RandomString(5) + Nonce()

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
// Ensure that the offset is greater than 0.
func GetTerms(search string, offset int, max int) ([]BannerTerm, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := BuildRequest("GET", "/classSearch/getTerms", map[string]string{
		"searchTerm": search,
		"offset":     strconv.Itoa(offset),
		"max":        strconv.Itoa(max),
		"_":          Nonce(),
	})

	if offset <= 0 {
		return nil, errors.New("Offset must be greater than 0")
	}

	res, err := DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get terms: %w", err)
	}

	// Assert that the response is JSON
	if contentType := res.Header.Get("Content-Type"); !strings.Contains(contentType, JsonContentType) {
		return nil, &UnexpectedContentTypeError{
			Expected: JsonContentType,
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
	json.Unmarshal(body, &terms)

	if err != nil {
		return nil, fmt.Errorf("failed to parse terms: %w", err)
	}

	return terms, nil
}

// SelectTerm selects the given term in the Banner system.
// This function completes the initial term selection process, which is required before any other API calls can be made with the session ID.
func SelectTerm(term string) {
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

	req := BuildRequestWithBody("POST", "/term/search", params, bytes.NewBufferString(form.Encode()))
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	res, err := DoRequest(req)
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Failed to select term")
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	// Acquire fwdUrl
	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Failed to read response body")
	}

	var redirectResponse struct {
		FwdUrl string `json:"fwdUrl"`
	}
	json.Unmarshal(body, &redirectResponse)

	// Make a GET request to the fwdUrl
	req = BuildRequest("GET", redirectResponse.FwdUrl, nil)
	res, err = DoRequest(req)
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Redirect request failed")
	}

	// Assert that the response is OK (200)
	if res.StatusCode != 200 {
		log.Fatal().Stack().Int("status", res.StatusCode).Msg("Unexpected status code from redirect request")
	}
}

// GetPartOfTerms retrieves and parses the part of term information for a given term.
// Ensure that the offset is greater than 0.
func GetPartOfTerms(search string, term int, offset int, max int) ([]BannerTerm, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := BuildRequest("GET", "/classSearch/get_partOfTerm", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get part of terms: %w", err)
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
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
func GetInstructors(search string, term string, offset int, max int) ([]Instructor, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := BuildRequest("GET", "/classSearch/get_instructor", map[string]string{
		"searchTerm":      search,
		"term":            term,
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get instructors: %w", err)
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
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

// TODO: Finish this struct & function
// ClassDetails represents
type ClassDetails struct {
}

func GetCourseDetails(term int, crn int) *ClassDetails {
	body, err := json.Marshal(map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
		"first":                 "first", // TODO: What is this?
	})
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Failed to marshal body")
	}
	req := BuildRequestWithBody("GET", "/searchResults/getClassDetails", nil, bytes.NewBuffer(body))

	res, err := DoRequest(req)
	if err != nil {
		return nil
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	return &ClassDetails{}
}

// Search invokes a search on the Banner system with the given query and returns the results.
func Search(query *Query, sort string, sortDescending bool) (*SearchResult, error) {
	ResetDataForm()

	params := query.Paramify()

	params["txt_term"] = "202420" // TODO: Make this automatic but dynamically specifiable
	params["uniqueSessionId"] = sessionID
	params["sortColumn"] = sort
	params["sortDirection"] = "asc"

	// These dates are not available for usage anywhere in the UI, but are included in every query
	params["startDatepicker"] = ""
	params["endDatepicker"] = ""

	req := BuildRequest("GET", "/searchResults/searchResults", params)

	res, err := DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to search: %w", err)
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Error().Stack().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	var result SearchResult
	err = json.Unmarshal(body, &result)

	if err != nil {
		return nil, fmt.Errorf("failed to parse search results: %w", err)
	}

	return &result, nil
}

// GetSubjects retrieves and parses the subject information for a given search term.
// The results of this response shouldn't change much, but technically could as new majors are developed, or old ones are removed.
// Ensure that the offset is greater than 0.
func GetSubjects(search string, term string, offset int, max int) ([]Pair, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := BuildRequest("GET", "/classSearch/get_subject", map[string]string{
		"searchTerm":      search,
		"term":            term,
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get subjects: %w", err)
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
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
func GetCampuses(search string, term int, offset int, max int) ([]Pair, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := BuildRequest("GET", "/classSearch/get_campus", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get campuses: %w", err)
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
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
func GetInstructionalMethods(search string, term string, offset int, max int) ([]Pair, error) {
	// Ensure offset is valid
	if offset <= 0 {
		return nil, errors.New("offset must be greater than 0")
	}

	req := BuildRequest("GET", "/classSearch/get_instructionalMethod", map[string]string{
		"searchTerm":      search,
		"term":            term,
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get instructional methods: %w", err)
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
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
func GetCourseMeetingTime(term int, crn int) ([]MeetingTimeResponse, error) {
	req := BuildRequest("GET", "/searchResults/getFacultyMeetingTimes", map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get meeting time: %w", err)
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
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
		Inner []MeetingTimeResponse `json:"fmt"`
	}
	err = json.Unmarshal(body, &meetingTime)
	if err != nil {
		return nil, fmt.Errorf("failed to parse meeting time: %w", err)
	}

	return meetingTime.Inner, nil
}

// ResetDataForm makes a POST request that needs to be made upon before new search requests can be made.
func ResetDataForm() {
	req := BuildRequest("POST", "/classSearch/resetDataForm", nil)
	_, err := DoRequest(req)
	if err != nil {
		log.Fatal().Stack().Err(err).Msg("Failed to reset data form")
	}
}

// GetCourse retrieves the course information.
// This course does not retrieve directly from the API, but rather uses scraped data stored in Redis.
func GetCourse(crn string) (*Course, error) {
	// Retrieve raw data
	result, err := kv.Get(ctx, fmt.Sprintf("class:%s", crn)).Result()
	if err != nil {
		if err == redis.Nil {
			return nil, fmt.Errorf("course not found: %w", err)
		}
		return nil, fmt.Errorf("failed to get course: %w", err)
	}

	// Unmarshal the raw data
	var course Course
	err = json.Unmarshal([]byte(result), &course)
	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal course: %w", err)
	}

	return &course, nil
}
