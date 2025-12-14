package main

import (
	"fmt"
	"reflect"
	tls "github.com/refraction-networking/utls"
)

func main() {
	t := reflect.TypeOf(tls.UtlsGREASEExtension{})
	fmt.Printf("UtlsGREASEExtension type: %v\n", t)
	for i := 0; i < t.NumField(); i++ {
		field := t.Field(i)
		fmt.Printf("Field %d: %s %v\n", i, field.Name, field.Type)
	}
}
