package config

import (
	"context"
	"net/http"
	"time"

	"github.com/redis/go-redis/v9"
)

type Config struct {
	Ctx                 context.Context
	KV                  *redis.Client
	Client              *http.Client
	IsDevelopment       bool
	BaseURL             string
	Environment         string
	CentralTimeLocation *time.Location
}

const (
	CentralTimezoneName = "America/Chicago"
)

func New() (*Config, error) {
	ctx := context.Background()

	loc, err := time.LoadLocation(CentralTimezoneName)
	if err != nil {
		return nil, err
	}

	return &Config{
		Ctx:                 ctx,
		CentralTimeLocation: loc,
	}, nil
}

// SetBaseURL sets the base URL for API requests
func (c *Config) SetBaseURL(url string) {
	c.BaseURL = url
}

// SetEnvironment sets the environment
func (c *Config) SetEnvironment(env string) {
	c.Environment = env
	c.IsDevelopment = env == "development"
}

// SetClient sets the HTTP client
func (c *Config) SetClient(client *http.Client) {
	c.Client = client
}

// SetRedis sets the Redis client
func (c *Config) SetRedis(r *redis.Client) {
	c.KV = r
}
