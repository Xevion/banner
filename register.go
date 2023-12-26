package main

import (
	"fmt"

	"github.com/bwmarrin/discordgo"
	"github.com/cespare/xxhash/v2"
	"github.com/cnf/structhash"
	"github.com/redis/go-redis/v9"
	log "github.com/rs/zerolog/log"
)

// Register registers all commands, and only registers commands that have changed.
// It also unregisters old commands if they are known to exist and have changed.
// This is to lower the cost of registering commands while iterating on the bot.
// In the case of an ambiguous state, all current commands are deleted while all new commands are registered.
func Register() ([]*discordgo.ApplicationCommand, error) {
	// Since we don't know what currentComands existed before, we can't unregister them if they're removed.
	// We also can't tell the difference between a command that was removed and one that was newly added.
	// Thus, if the number of currently active currentComands is not equal to the number of currentComands we're registering, we'll just re-register all of them.
	currentComands, err := session.ApplicationCommands(session.State.User.ID, "")
	if err != nil {
		log.Panic().Err(err).Msg("Cannot get commands")
	}
	if len(currentComands) != len(commandDefinitions) {
		log.Info().Int("registered", len(currentComands)).Int("definitions", len(commandDefinitions)).Msg("Number of registered commands does not match number of command definitions, registering all commands")
		return SimpleRegister(currentComands)
	}

	registeredCommands := make([]*discordgo.ApplicationCommand, len(commandDefinitions))
	for i, cmdDefinition := range commandDefinitions {
		// Create a hash of the command definition
		hash := xxhash.Sum64(structhash.Dump(cmdDefinition, 1))
		key := fmt.Sprintf("%s:command:%s:xxhash", environment, cmdDefinition.Name)

		// Get the stored hash
		storedHash, err := kv.Get(ctx, key).Uint64()
		if err != nil {
			if err != redis.Nil {
				log.Err(err).Msg("Cannot get command hash from redis")
			} else {
				log.Debug().Str("command", cmdDefinition.Name).Str("key", key).Msg("Command hash not found in redis")
			}
		}

		// If the hash is the same, skip registering the command
		if hash == storedHash {
			log.Debug().Str("command", cmdDefinition.Name).Str("key", key).Uint64("hash", hash).Msg("Command hash matches, skipping registration")
			continue
		}
		log.Info().Str("command", cmdDefinition.Name).Str("key", key).Uint64("hash", hash).Uint64("storedHash", storedHash).Msg("Command hash mismatch, registering command")

		// Unregister the old command first (retrieve the ID from redis)
		oldCommandId, err := kv.Get(ctx, fmt.Sprintf("%s:command:%s:id", environment, cmdDefinition.Name)).Result()
		if err != nil {
			if err != redis.Nil {
				// Not really sure what to do here, something failed with redis. Best to just keep the old commad in place.
				log.Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot get old command ID from redis (skipping registration/deletion)")
				continue
			} else {
				// It's an unlikely case, but if required, we could get the old command ID from discord to unregisstter it.
				log.Debug().Str("name", cmdDefinition.Name).Str("key", key).Msg("Old command ID not found in redis")
			}
		} else {
			err = session.ApplicationCommandDelete(session.State.User.ID, "", oldCommandId)
			if err != nil {
				// No panic - the command is probably still registered.
				log.Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot unregister old command")
				continue
			} else {
				log.Info().Str("name", cmdDefinition.Name).Str("key", key).Msg("Unregistered old command")
			}
		}

		// Register the command
		cmdInstance, err := session.ApplicationCommandCreate(session.State.User.ID, "", cmdDefinition)
		if err != nil {
			log.Panic().Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot register command")
		}
		registeredCommands[i] = cmdInstance
		log.Info().Str("name", cmdDefinition.Name).Str("key", key).Msg("Registered command")

		// Store the hash for the new registered command
		err = kv.Set(ctx, key, hash, 0).Err()
		if err != nil {
			// No panic - the command is still registered, hash is only to prevent unnecessary registrations
			log.Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot set command hash in redis")
			continue
		}

		// Store the command ID to unregister later
		err = kv.Set(ctx, fmt.Sprintf("%s:command:%s:id", environment, cmdDefinition.Name), cmdInstance.ID, 0).Err()
		if err != nil {
			// No
			log.Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot set command ID in redis")
		}
	}

	return registeredCommands, nil
}

func SimpleRegister(currentCommands []*discordgo.ApplicationCommand) ([]*discordgo.ApplicationCommand, error) {
	registeredCommands := make([]*discordgo.ApplicationCommand, len(commandDefinitions))

	// Unregister all commands
	for _, cmd := range currentCommands {
		err := session.ApplicationCommandDelete(session.State.User.ID, "", cmd.ID)
		if err != nil {
			log.Err(err).Str("name", cmd.Name).Msg("Cannot unregister command")
		} else {
			log.Info().Str("name", cmd.Name).Msg("Unregistered command")
		}
	}

	// Register all commands
	for i, cmdDefinition := range commandDefinitions {
		// Create a hash of the command definition
		hash := xxhash.Sum64(structhash.Dump(cmdDefinition, 1))
		key := fmt.Sprintf("%s:command:%s:xxhash", environment, cmdDefinition.Name)

		cmdInstance, err := session.ApplicationCommandCreate(session.State.User.ID, "", cmdDefinition)
		if err != nil {
			log.Panic().Err(err).Str("name", cmdDefinition.Name).Msg("Cannot register command")
		}
		registeredCommands[i] = cmdInstance
		log.Info().Str("name", cmdDefinition.Name).Msg("Registered command")

		// Store the hash for the new registered command
		err = kv.Set(ctx, key, hash, 0).Err()
		if err != nil {
			// No panic - the command is still registered, hash is only to prevent unnecessary registrations
			log.Err(err).Str("name", cmdDefinition.Name).Str("key", key).Msg("Cannot set command hash in redis")
			continue
		}
	}
	return registeredCommands, nil
}
