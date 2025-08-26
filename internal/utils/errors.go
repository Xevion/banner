package utils

import "fmt"

type UnexpectedContentTypeError struct {
	Expected string
	Actual   string
}

func (e *UnexpectedContentTypeError) Error() string {
	return fmt.Sprintf("Expected content type '%s', received '%s'", e.Expected, e.Actual)
}
