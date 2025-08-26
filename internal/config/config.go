package config

import (
	"context"
	"net/http"
	"time"

	"github.com/redis/go-redis/v9"
)

var (
	// Global variables that need to be accessible across packages
	Ctx                 context.Context
	KV                  *redis.Client
	Client              http.Client
	Cookies             http.CookieJar
	IsDevelopment       bool
	BaseURL             string
	Environment         string
	CentralTimeLocation *time.Location
	IsClosing           bool = false
)

const (
	ICalTimestampFormatUtc   = "20060102T150405Z"
	ICalTimestampFormatLocal = "20060102T150405"
	CentralTimezoneName      = "America/Chicago"
)

func init() {
	Ctx = context.Background()

	var err error
	CentralTimeLocation, err = time.LoadLocation(CentralTimezoneName)
	if err != nil {
		panic(err)
	}
}

// SetBaseURL sets the base URL for API requests
func SetBaseURL(url string) {
	BaseURL = url
}

// SetEnvironment sets the environment
func SetEnvironment(env string) {
	Environment = env
	IsDevelopment = env == "development"
}

// SetClient sets the HTTP client
func SetClient(c http.Client) {
	Client = c
}

// SetCookies sets the cookie jar
func SetCookies(cj http.CookieJar) {
	Cookies = cj
}

// SetRedis sets the Redis client
func SetRedis(r *redis.Client) {
	KV = r
}
