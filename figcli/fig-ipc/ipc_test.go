package fig_ipc

import "testing"

func TestConnectClose(t *testing.T) {
	conn, err := Connect()
	if err != nil {
		t.Errorf("Connect() failed: %s", err)
	}

	err = conn.Close()
	if err != nil {
		t.Errorf("Close() failed: %s", err)
	}

	err = conn.Close()
	if err == nil {
		t.Errorf("Close() should have failed")
	}
}
