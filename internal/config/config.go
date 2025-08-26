package config

import (
	"context"
	"time"

	"github.com/redis/go-redis/v9"
	"resty.dev/v3"
)

type Config struct {
	Ctx                 context.Context
	CancelFunc          context.CancelFunc
	KV                  *redis.Client
	Client              *resty.Client
	IsDevelopment       bool
	BaseURL             string
	Environment         string
	CentralTimeLocation *time.Location
}

const (
	CentralTimezoneName = "America/Chicago"
)

func New() (*Config, error) {
	ctx, cancel := context.WithCancel(context.Background())

	loc, err := time.LoadLocation(CentralTimezoneName)
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

// SetBaseURL sets the base URL for API requests
func (c *Config) SetBaseURL(url string) {
	c.BaseURL = url
}

// SetEnvironment sets the environment
func (c *Config) SetEnvironment(env string) {
	c.Environment = env
	c.IsDevelopment = env == "development"
}

// SetClient sets the Resty client
func (c *Config) SetClient(client *resty.Client) {
	c.Client = client
}

// SetRedis sets the Redis client
func (c *Config) SetRedis(r *redis.Client) {
	c.KV = r
}
