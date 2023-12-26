package main

import (
	"net/url"

	log "github.com/rs/zerolog/log"
)

func setup() {
	// Makes the initial requests that sets up the session cookies for the rest of the application
	log.Info().Msg("Setting up session...")

	request_queue := []string{
		"/registration/registration",
		"/selfServiceMenu/data",
	}

	for _, path := range request_queue {
		req := BuildRequest("GET", path, nil)
		DoRequest(req)
	}

	// Validate that cookies were set
	baseUrlParsed, err := url.Parse(baseURL)
	if err != nil {
		log.Fatal().Str("baseURL", baseURL).Err(err).Msg("Failed to parse baseURL")
	}

	current_cookies := client.Jar.Cookies(baseUrlParsed)
	required_cookies := map[string]bool{
		"JSESSIONID": false,
		"SSB_COOKIE": false,
	}

	for _, cookie := range current_cookies {
		_, present := required_cookies[cookie.Name]
		// Check if this cookie is required
		if present {
			required_cookies[cookie.Name] = true
		}
	}

	// Check if all required cookies were set
	for cookieName, cookie_set := range required_cookies {
		if !cookie_set {
			log.Warn().Str("cookieName", cookieName).Msg("Required cookie not set")
		}
	}
	log.Debug().Msg("All required cookies set, session setup complete")

	// TODO: Validate that the session allows access to termSelection
}
