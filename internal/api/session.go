package api

import (
	"banner/internal/config"
	"banner/internal/utils"
	"net/url"

	log "github.com/rs/zerolog/log"
)

func Setup() {
	// Makes the initial requests that sets up the session cookies for the rest of the application
	log.Info().Msg("Setting up session...")

	requestQueue := []string{
		"/registration/registration",
		"/selfServiceMenu/data",
	}

	for _, path := range requestQueue {
		req := utils.BuildRequest("GET", path, nil)
		DoRequest(req)
	}

	// Validate that cookies were set
	baseURLParsed, err := url.Parse(config.BaseURL)
	if err != nil {
		log.Fatal().Stack().Str("baseURL", config.BaseURL).Err(err).Msg("Failed to parse baseURL")
	}

	currentCookies := config.Client.Jar.Cookies(baseURLParsed)
	requiredCookies := map[string]bool{
		"JSESSIONID": false,
		"SSB_COOKIE": false,
	}

	for _, cookie := range currentCookies {
		_, present := requiredCookies[cookie.Name]
		// Check if this cookie is required
		if present {
			requiredCookies[cookie.Name] = true
		}
	}

	// Check if all required cookies were set
	for cookieName, cookieSet := range requiredCookies {
		if !cookieSet {
			log.Warn().Str("cookieName", cookieName).Msg("Required cookie not set")
		}
	}
	log.Debug().Msg("All required cookies set, session setup complete")

	// TODO: Validate that the session allows access to termSelection
}
