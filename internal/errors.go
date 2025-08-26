package internal

import "fmt"

// UnexpectedContentTypeError is returned when the Content-Type header of a response does not match the expected value.
type UnexpectedContentTypeError struct {
	Expected string
	Actual   string
}

func (e *UnexpectedContentTypeError) Error() string {
	return fmt.Sprintf("Expected content type '%s', received '%s'", e.Expected, e.Actual)
}
