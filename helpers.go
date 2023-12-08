package banner

import (
	"net/http"
	"strings"
)

func buildURL(path string, params map[string]string) string {
	// Builds a URL for the given path and parameters
	url := baseURL + path

	if params != nil {
		takenFirst := false
		for key, value := range params {
			paramChar := "&"
			if !takenFirst {
				paramChar = "?"
				takenFirst = true
			}

			url += paramChar + key + "=" + value
		}
	}

	return url
}

func AddUserAgent(req *http.Request) {
	req.Header.Add("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
}

func ContainsContentType(header string, search string) bool {
	// Split on commas, check if any of the types match
	for _, content_type := range strings.Split(header, ";") {
		if content_type == search {
			return true
		}
	}
	return false
}
