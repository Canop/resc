package main

import (
	"fmt"
	"github.com/garyburd/redigo/redis"
	"log"
	"strings"
	"time"
)

const REDIS_URL = "redis://127.0.0.1"
const INPUT_QUEUE = "trt/plantA/todo"
const TAKEN_QUEUE = "trt/plantA/taken"
const OUTPUT_QUEUE = "global/done"

func handleTask(task string) {
	tokens := strings.Split(task, "/")
	if len(tokens) != 3 {
		log.Printf("Illegal task : %v", task)
		return
	}
	nature, process, product := tokens[0], tokens[1], tokens[2]
	fmt.Printf("Executing %s for product %s on process %s ", nature, product, process)
	for i := 0; i < 10; i++ {
		time.Sleep(time.Second)
		fmt.Print(".")
	}
	log.Println(" done")
}

func main() {
	con, err := redis.DialURL(REDIS_URL)
	if err != nil {
		log.Fatalf("Could not connect: %v\n", err)
	}
	defer con.Close()
	log.Printf("Worker listening on queue %+v\n", INPUT_QUEUE)
	for {
		task, _ := redis.String(con.Do("BRPOPLPUSH", INPUT_QUEUE, TAKEN_QUEUE, 60))
		if task != "" {
			handleTask(task)
			if _, err = con.Do("LPUSH", OUTPUT_QUEUE, task); err != nil {
				log.Fatalf("Error in LPUSH: %v\n", err)
			}
			if _, err = con.Do("LREM", TAKEN_QUEUE, 1, task); err != nil {
				log.Fatalf("Error in LREM: %v\n", err)
			}
		} else {
			log.Println("I'm bored")
		}
	}
}
