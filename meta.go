package main

import (
	"time"

	"github.com/redis/go-redis/v9"
	log "github.com/rs/zerolog/log"
)

// GetGuildName returns the name of the guild with the given ID, utilizing Redis to cache the value
func GetGuildName(guildID string) string {
	// Check Redis for the guild name
	guildName, err := kv.Get(ctx, "guild:"+guildID+":name").Result()
	if err != nil && err != redis.Nil {
		log.Error().Err(err).Msg("Error getting guild name from Redis")
		return "err"
	}

	// If the guild name is invalid (1 character long), then return "unknown"
	if len(guildName) == 1 {
		return "unknown"
	}

	// If the guild name isn't in Redis, get it from Discord and cache it
	guild, err := session.Guild(guildID)
	if err != nil {
		log.Error().Err(err).Msg("Error getting guild name")

		// Store an invalid value in Redis so we  don't keep trying to get the guild name
		_, err := kv.Set(ctx, "guild:"+guildID+":name", "x", time.Minute*5).Result()
		if err != nil {
			log.Error().Err(err).Msg("Error setting false guild name in Redis")
		}

		return "unknown"
	}

	// Cache the guild name in Redis
	kv.Set(ctx, "guild:"+guildID+":name", guild.Name, time.Hour*3)

	return guild.Name
}

// GetChannelName returns the name of the channel with the given ID, utilizing Redis to cache the value
func GetChannelName(channelID string) string {
	// Check Redis for the channel name
	channelName, err := kv.Get(ctx, "channel:"+channelID+":name").Result()
	if err != nil && err != redis.Nil {
		log.Error().Err(err).Msg("Error getting channel name from Redis")
		return "err"
	}

	// If the channel name is invalid (1 character long), then return "unknown"
	if len(channelName) == 1 {
		return "unknown"
	}

	// If the channel name isn't in Redis, get it from Discord and cache it
	channel, err := session.Channel(channelID)
	if err != nil {
		log.Error().Err(err).Msg("Error getting channel name")

		// Store an invalid value in Redis so we  don't keep trying to get the channel name
		_, err := kv.Set(ctx, "channel:"+channelID+":name", "x", time.Minute*5).Result()
		if err != nil {
			log.Error().Err(err).Msg("Error setting false channel name in Redis")
		}

		return "unknown"
	}

	// Cache the channel name in Redis
	kv.Set(ctx, "channel:"+channelID+":name", channel.Name, time.Hour*3)

	return channel.Name
}
