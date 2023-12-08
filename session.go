package banner

import (
	"log"
	"net/url"
)

func setup() {
	// Makes the initial requests that sets up the session cookies for the rest of the application
	log.Println("Setting up session...")

	request_queue := []string{
		"/registration/registration",
		"/selfServiceMenu/data",
	}

	for _, path := range request_queue {
		req := BuildRequest("GET", path, nil)
		log.Printf("GET %s", req.URL.String())
		res, _ := client.Do(req)
		log.Printf("%s %s", res.Status, res.Header["Content-Type"])
	}

	// Validate that cookies were set
	baseURL_parsed, _ := url.Parse(baseURL)
	current_cookies := cookies.Cookies(baseURL_parsed)
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
	for cookie_name, cookie_set := range required_cookies {
		if !cookie_set {
			log.Fatalf("Required cookie %s was not set", cookie_name)
		}
	}
	log.Println("All cookies acquired. Session setup complete.")

	// TODO: Validate that the session allows access to termSelection
}
