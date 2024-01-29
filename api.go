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

	log "github.com/rs/zerolog/log"
)

var sessionID string = RandomString(5) + Nonce()

// BannerTerm represents a term in the Banner system
type BannerTerm struct {
	Code        string `json:"code"`
	Description string `json:"description"`
}

// GetTerms retrieves and parses the term information for a given search term.
func GetTerms(search string, offset int, max int) ([]BannerTerm, error) {
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
		return nil, err
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
	body, _ := io.ReadAll(res.Body)

	terms := make([]BannerTerm, 0, 10)
	json.Unmarshal(body, &terms)

	if err != nil {
		return nil, err
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
		log.Fatal().Err(err).Msg("Failed to select term")
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	// Acquire fwdUrl
	defer res.Body.Close()
	body, _ := io.ReadAll(res.Body)
	var redirectResponse struct {
		FwdUrl string `json:"fwdUrl"`
	}
	json.Unmarshal(body, &redirectResponse)

	// Make a GET request to the fwdUrl
	req = BuildRequest("GET", redirectResponse.FwdUrl, nil)
	res, err = DoRequest(req)
	if err != nil {
		log.Fatal().Err(err).Msg("Redirect request failed")
	}

	// Assert that the response is OK (200)
	if res.StatusCode != 200 {
		log.Fatal().Int("status", res.StatusCode).Msg("Unexpected status code from redirect request")
	}
}

// GetPartOfTerms retrieves and parses the part of term information for a given term.
func GetPartOfTerms(search string, term int, offset int, max int) ([]BannerTerm, error) {
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
		log.Fatal().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	defer res.Body.Close()
	body, _ := io.ReadAll(res.Body)

	terms := make([]BannerTerm, 0, 10)
	err = json.Unmarshal(body, &terms)
	if err != nil {
		return nil, fmt.Errorf("failed to parse part of terms: %w", err)
	}

	return terms, nil
}

type Instructor struct {
	Code        string `json:"code"`
	Description string `json:"description"`
}

// GetInstructors retrieves and parses the instructor information for a given search term.
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
		log.Fatal().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	defer res.Body.Close()
	body, _ := io.ReadAll(res.Body)

	instructors := make([]Instructor, 0, 10)
	err = json.Unmarshal(body, &instructors)
	if err != nil {
		return nil, fmt.Errorf("failed to parse instructors: %w", err)
	}

	return instructors, nil
}

// TODO: Finish this struct & function
type ClassDetails struct {
}

func GetClassDetails(term int, crn int) *ClassDetails {
	body, _ := json.Marshal(map[string]string{
		"term":                  strconv.Itoa(term),
		"courseReferenceNumber": strconv.Itoa(crn),
		"first":                 "first", // TODO: What is this?
	})
	req := BuildRequestWithBody("GET", "/searchResults/getClassDetails", nil, bytes.NewBuffer(body))

	res, err := DoRequest(req)
	if err != nil {
		return nil
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	return &ClassDetails{}
}

// Search invokes a search on the Banner system with the given query and returns the results.
func Search(query *Query, sort string, sortDescending bool) (*SearchResult, error) {
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
		return nil, err
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Error().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	body, _ := io.ReadAll(res.Body)

	var result SearchResult
	err = json.Unmarshal(body, &result)

	if err != nil {
		return nil, fmt.Errorf("failed to parse search results: %w", err)
	}

	return &result, nil
}

// TODO: Finish this struct & function
type Subject struct{}

func GetSubjects(search string, term int, offset int, max int) []Subject {
	req := BuildRequest("GET", "/classSearch/get_subject", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		return nil
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	return make([]Subject, 0)
}

// TODO: Finish this struct & function
type Campus struct{}

func GetCampuses(search string, term int, offset int, max int) []Campus {
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
		log.Err(err)
		return nil
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	return make([]Campus, 0)
}

// TODO: Finish this struct & function
type InstructionalMethod struct{}

func GetInstructionalMethods(search string, term int, offset int, max int) ([]InstructionalMethod, error) {
	req := BuildRequest("GET", "/classSearch/get_instructionalMethod", map[string]string{
		"searchTerm":      search,
		"term":            strconv.Itoa(term),
		"offset":          strconv.Itoa(offset),
		"max":             strconv.Itoa(max),
		"uniqueSessionId": sessionID,
		"_":               Nonce(),
	})

	res, err := DoRequest(req)
	if err != nil {
		log.Err(err).Msg("Failed to get instructional methods")
		return nil, err
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	return make([]InstructionalMethod, 0), nil
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
		return nil, err
	}

	// Assert that the response is JSON
	if !ContentTypeMatch(res, "application/json") {
		log.Fatal().Str("content-type", res.Header.Get("Content-Type")).Msg("Response was not JSON")
	}

	// Read the response body into JSON
	defer res.Body.Close()
	body, _ := io.ReadAll(res.Body)

	// Parse the JSON into a MeetingTimeResponse struct
	var meetingTime struct {
		Inner []MeetingTimeResponse `json:"fmt"`
	}
	err = json.Unmarshal(body, &meetingTime)
	if err != nil {
		return nil, err
	}

	return meetingTime.Inner, nil
}
