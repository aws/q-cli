package fig_ipc

import (
	"bytes"
	"context"
	"encoding/binary"
	"fmt"
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

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*5)
	defer cancel()

	raddr := net.UnixAddr{Name: socket, Net: "unix"}
	conn, err := d.DialContext(ctx, "unix", raddr.String())

	if err != nil {
		return nil, fmt.Errorf("unable to connect to socket: %s", err)
	}

	return &FigIpc{conn: conn.(*net.UnixConn)}, nil
}

// Close the connection
func (f *FigIpc) Close() error {
	return f.conn.Close()
}

// Send fig-json to the server
func (f *FigIpc) SendFigJson(msg string) error {
	buf := new(bytes.Buffer)

	if _, err := buf.Write([]byte("\x1b@fig-json")); err != nil {
		return err
	}

	// Write the size of the message
	if err := binary.Write(buf, binary.BigEndian, uint64(len(msg))); err != nil {
		return err
	}

	if _, err := f.conn.Write([]byte(msg)); err != nil {
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

	buf := new(bytes.Buffer)

	if _, err := buf.Write([]byte("\x1b@fig-pbuf")); err != nil {
		return err
	}

	// Write the size of the message
	if err := binary.Write(buf, binary.BigEndian, uint64(len(data))); err != nil {
		return err
	}

	if _, err = buf.Write(data); err != nil {
		return err
	}

	f.conn.Write(buf.Bytes())

	return nil
}

type ProtoType int

const (
	protoTypeUndefined ProtoType = iota
	protoTypeFigJson
	protoTypeFigProto
)

type MessageResponse struct {
	Message   []byte
	ProtoType ProtoType
	Error     error
}

func (f *FigIpc) RecvMessage() MessageResponse {
	// Read first 10 bytes to get the type
	buf := make([]byte, 10)
	if _, err := f.conn.Read(buf); err != nil {
		return MessageResponse{Error: err}
	}

	// Determine the type of the message
	protoType := protoTypeUndefined
	switch string(buf) {
	case "\x1b@fig-json":
		protoType = protoTypeFigJson
	case "\x1b@fig-pbuf":
		protoType = protoTypeFigProto
	}

	if protoType == protoTypeUndefined {
		return MessageResponse{Error: fmt.Errorf("unknown message type: %s", buf[1:])}
	}

	// Read u64 from the stream to determine the size of the message
	var size uint64
	if err := binary.Read(f.conn, binary.BigEndian, &size); err != nil {
		return MessageResponse{Error: err}
	}

	// Read the rest of the data
	buf = make([]byte, size)
	if _, err := f.conn.Read(buf); err != nil {
		return MessageResponse{Error: err}
	}

	return MessageResponse{Message: buf, ProtoType: protoType}
}

func (f *FigIpc) RecvMessageTimeout(duration time.Duration) MessageResponse {
	channel := make(chan MessageResponse, 1)
	go func() {
		msg := f.RecvMessage()
		channel <- msg
	}()

	select {
	case res := <-channel:
		return res
	case <-time.After(duration):
		return MessageResponse{Error: fmt.Errorf("timeout waiting for message")}
	}
}
