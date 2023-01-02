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

		mode := 4
		if mode == 1 {
			w.WriteHeader(200)
			w.Write([]byte(""))
		} else if mode == 2 {
			w.WriteHeader(500)
			w.Write([]byte("error: 429"))
		} else if mode == 3 {
			w.WriteHeader(500)
			w.Write([]byte("error: notEnoughPoints"))
		} else if mode == 4 {
			w.WriteHeader(400)
			w.Write([]byte("error: some error"))
		} else {
			panic("The value of `mode` is invalid.")
		}
	})

	fmt.Println("Started listening the port 8080...")
	server.ListenAndServe()

}
