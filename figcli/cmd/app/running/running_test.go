package running

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func Test_running(t *testing.T) {
	assert.Containsf(t, []string{"1", "0"}, running(), "figcli is running", "should print running")
}
