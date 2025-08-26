package config

import (
	"context"
	"time"

	"github.com/redis/go-redis/v9"
	"resty.dev/v3"
)

// Config holds the application's configuration.
type Config struct {
	// Ctx is the application's root context.
	Ctx context.Context
	// CancelFunc cancels the application's root context.
	CancelFunc context.CancelFunc
	// KV provides access to the Redis cache.
	KV *redis.Client
	// Client is the HTTP client for making API requests.
	Client *resty.Client
	// IsDevelopment is true if the application is running in a development environment.
	IsDevelopment bool
	// BaseURL is the base URL for the Banner API.
	BaseURL string
	// Environment is the application's running environment (e.g. "development").
	Environment string
	// CentralTimeLocation is the time.Location for US Central Time.
	CentralTimeLocation *time.Location
}

const (
	CentralTimezoneName = "America/Chicago"
)

// New creates a new Config instance with a cancellable context.
func New() (*Config, error) {
	ctx, cancel := context.WithCancel(context.Background())

	loc, err := time.LoadLocation("America/Chicago")
	if err != nil {
		cancel()
		return nil, err
	}

	return &Config{
		Ctx:                 ctx,
		CancelFunc:          cancel,
		CentralTimeLocation: loc,
	}, nil
}

// SetBaseURL sets the base URL for the Banner API.
func (c *Config) SetBaseURL(url string) {
	c.BaseURL = url
}

// SetEnvironment sets the application's environment.
func (c *Config) SetEnvironment(env string) {
	c.Environment = env
	c.IsDevelopment = env == "development"
}

// SetClient sets the Resty client for making HTTP requests.
func (c *Config) SetClient(client *resty.Client) {
	c.Client = client
}

// SetRedis sets the Redis client for caching.
func (c *Config) SetRedis(r *redis.Client) {
	c.KV = r
}
