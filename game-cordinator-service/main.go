package main

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/redis/go-redis/v9"
)

var servers = []string{"127.0.0.1:7878", "127.0.0.1:7879"}
var ctx = context.Background()
var channel = "game_servers"

type GameServer struct {
	GameID        string `json:"game_id"`
	ServerAddress string `json:"server_address"`
}

func main() {
	rdb := redis.NewClient(&redis.Options{
		Addr:     "127.0.0.1:6379",
		Password: "",
		DB:       0,
	})

	http.HandleFunc("/assign_server", func(w http.ResponseWriter, r *http.Request) {
		assignServerHandler(w, r, rdb)
	})

	fmt.Print("Starting server on :8081")

	if err := http.ListenAndServe(":8081", nil); err != nil {
		fmt.Printf("Error starting server: %s\n", err)
	}
}

func assignServerHandler(w http.ResponseWriter, r *http.Request, db *redis.Client) {
	lobbyID := r.URL.Query().Get("lobby_id")

	if lobbyID == "" {
		http.Error(w, "Error reading request body", http.StatusInternalServerError)
		return
	}

	res, err := db.Get(ctx, lobbyID).Result()

	if err != nil {
		server, err := findServerWithLowestEntries(db)
		if err != nil {
			http.Error(w, "Error reading request body", http.StatusInternalServerError)
			return
		}

		db.Set(ctx, lobbyID, server, 0)
		gameServer := GameServer{GameID: lobbyID, ServerAddress: server}
		data, err := json.Marshal(gameServer)

		if err == nil {
			db.Publish(ctx, channel, data)
		} else {
			fmt.Printf("Error during serialization %s\n", err)
		}

		fmt.Fprint(w, server)
	} else {
		fmt.Fprint(w, res)
	}
}

func removeServer() {
	// TODO
}

func findServerWithLowestEntries(rdb *redis.Client) (string, error) {
	var minServer string
	minCount := int64(-1)

	for _, server := range servers {
		iter := rdb.Scan(ctx, 0, "*", 0).Iterator()
		count := int64(0)
		for iter.Next(ctx) {
			key := iter.Val()
			val, err := rdb.Get(ctx, key).Result()
			if err == nil && val == server {
				count++
			}
		}
		if err := iter.Err(); err != nil {
			return "", err
		}

		if minCount == -1 || count < minCount {
			minCount = count
			minServer = server
		}
	}

	return minServer, nil
}
