package main

import (
	"github.com/tidwall/redcon"
	"log"
	"strings"
)

func CreateRedconServer(port string) error {
	return redcon.ListenAndServe(":"+port,
		func(conn redcon.Conn, cmd redcon.Command) {
			command := strings.ToLower(string(cmd.Args[0]))
			var argsStr []string
			for _, arg := range cmd.Args {
				argsStr = append(argsStr, string(arg))
			}
			log.Printf("Args (%s): %+v", conn.Context(), strings.Join(argsStr, " "))
			switch command {
			case "set":
				conn.WriteString("a")
			case "get":
				conn.WriteString("a")
			case "hello":
				conn.WriteArray(2)
				conn.WriteString("1")
				conn.WriteString("2")
			default:
				conn.WriteNull()
			}
		},
		func(conn redcon.Conn) bool {
			// Use this function to accept or deny the connection.
			// log.Printf("accept: %s", conn.RemoteAddr())
			conn.SetContext("hey") // this is like an ID
			return true
		},
		func(conn redcon.Conn, err error) {
			// This is called when the connection has been closed
			// log.Printf("closed: %s, err: %v", conn.RemoteAddr(), err)
		},
	)
}
