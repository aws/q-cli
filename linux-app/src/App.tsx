import type { Component } from 'solid-js';
import {createEffect, createSignal} from 'solid-js';
import {listen} from '@tauri-apps/api/event';
import {createStore} from "solid-js/store";

interface EditBuffer {
  text: string,
  idx: number,
}

interface CursorPosition {
  x: number,
  y: number,
  width: number,
  height: number,
}

const App: Component = () => {
  const [editBuffer, setEditBuffer] = createSignal({
    text: "",
    idx: 0,
  });

  listen('update-edit-buffer', event => {
    setEditBuffer(event.payload as EditBuffer);
  });

  const [cursorPosition, setCursorPosition] = createSignal({
    x: 0,
    y: 0,
    width: 0,
    height: 0,
  });

  listen('update-cursor-position', event => {
    setCursorPosition(event.payload as CursorPosition);
  });

  createEffect(() => {
    const cursor = document.getElementById("cursor");
    console.log(editBuffer().idx);
  })

  return (
    <div class="p-10">
      <input class="mx-auto p-2 pb-1 w-full bg-black text-white font-mono" value={editBuffer().text} />
      <div id="cursor" class="w-2 h-5 bg-white absolute top-12" style={`left: ${48 + editBuffer().idx * 10}px`} />
      <p>Cursor at:<br/>x: {cursorPosition().x}<br/>y: {cursorPosition().y}<br/>width: {cursorPosition().width}<br/>height: {cursorPosition().height}</p>
    </div>
  );
};

export default App;
