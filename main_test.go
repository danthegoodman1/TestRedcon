package main

import (
	"context"
	"log"
	"testing"

	"github.com/redis/go-redis/v9"
)

func TestPipeline(t *testing.T) {
	go CreateRedconServer("6379")
	r := redis.NewClient(&redis.Options{
		Addr:     "localhost:6379",
		Password: "e",
		Username: "username",
	})
	ctx := context.Background()
	cmds, err := r.Pipelined(ctx, func(pipeliner redis.Pipeliner) error {
		pipeliner.Set(ctx, "setkey1", "setval1", 0)
		pipeliner.Set(ctx, "setkey2", "setval2", 0)
		pipeliner.Get(ctx, "getkey")
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}

	for _, cmd := range cmds {
		if c, ok := cmd.(*redis.StringCmd); ok {
			log.Println("Command", c.Args(), "result:", c.Val())
		}
	}

	r.Close()
}
