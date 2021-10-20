package fig_ipc

import (
	"context"
	"net"
	"os"
	"time"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type FigIpc struct {
	conn *net.UnixConn
}

// Connect to the server
func Connect() (*FigIpc, error) {
	socket := os.Getenv("TMPDIR") + "fig.socket"

	var d net.Dialer
	ctx, cancel := context.WithTimeout(context.Background(), time.Minute)
	defer cancel()

	raddr := net.UnixAddr{Name: socket, Net: "unix"}
	conn, err := d.DialContext(ctx, "unix", raddr.String())

	if err != nil {
		return nil, err
	}

	return &FigIpc{conn: conn.(*net.UnixConn)}, nil
}

// Close the connection
func (f *FigIpc) Close() error {
	return f.conn.Close()
}

// Send a message to the server
func (f *FigIpc) Send(msg string) error {
	_, err := f.conn.Write([]byte(msg))
	return err
}

// Receive a message from the server, reading until a newline
func (f *FigIpc) Recv() (string, error) {
	buf := make([]byte, 1)
	var msg []byte
	for {
		_, err := f.conn.Read(buf)
		if err != nil {
			return "", err
		}
		if buf[0] == '\n' {
			break
		}
		msg = append(msg, buf[0])
	}
	return string(msg), nil
}

// Send fig-json to the server
func (f *FigIpc) SendFigJson(msg string) error {
	_, err := f.conn.Write([]byte("\x1b@fig-json"))
	if err != nil {
		return err
	}

	_, err = f.conn.Write([]byte(msg))
	if err != nil {
		return err
	}

	_, err = f.conn.Write([]byte("\x1b\\"))
	if err != nil {
		return err
	}

	return nil
}

// Send fig-proto to the server
func (f *FigIpc) SendFigProto(m protoreflect.ProtoMessage) error {
	data, err := proto.Marshal(m)
	if err != nil {
		return err
	}

	_, err = f.conn.Write([]byte("\x1b@fig-proto"))
	if err != nil {
		return err
	}

	_, err = f.conn.Write(data)
	if err != nil {
		return err
	}

	_, err = f.conn.Write([]byte("\x1b\\"))
	if err != nil {
		return err
	}

	return nil
}

func (f *FigIpc) RecvData() (string, error) {
	// Read data until escape backslash
	buf := make([]byte, 1)
	prev_char := byte(0)
	var msg []byte
	for {
		_, err := f.conn.Read(buf)
		if err != nil {
			return "", err
		}
		if buf[0] == '\x1b' && prev_char == '\\' {
			break
		}
		msg = append(msg, buf[0])
		prev_char = buf[0]
	}

	// Parse the message type
	// msg_type := string(msg[])
	// if msg_type != "fig-json" && msg_type != "fig-proto" {
	// 	return "", nil
	// }

	return string(msg), nil
}
