package main

import "net/http"
import "fmt"
import "time"

func main() {

	var server = &http.Server{
		Addr:         ":8080",
		ReadTimeout:  5 * time.Second,
		WriteTimeout: 1 * time.Second,
	}

	http.HandleFunc("/", func(w http.ResponseWriter, req *http.Request) {
		fmt.Println("----------")
		fmt.Println(req)
		w.WriteHeader(200)
		w.Write([]byte("OK"))
	})

	fmt.Println("Started listening the port 8080...")
	server.ListenAndServe()

}
