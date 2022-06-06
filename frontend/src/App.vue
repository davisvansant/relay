<script setup lang="ts">
import { nextTick, reactive, ref, watch } from 'vue'

const url = ref('ws://localhost:1806/ws')
const connection = new WebSocket(url.value)

interface Message {
  id: number,
  text: string,
}

const messages:Message[] = reactive([])
const uuid = ref('')
const connected_users = ref(0)
const ready_state = ref('')

connection.addEventListener('message', (MessageEvent) => {
      console.log('Received message ->', MessageEvent)

      checkReadyState()

      const IncomingMessage = JSON.parse(MessageEvent.data)

      switch (IncomingMessage.kind) {
        case 'connected_users':
          connected_users.value = IncomingMessage.contents
          break
        case 'message':
          receiveMessage(IncomingMessage.contents)
          break
        case 'uuid':
          uuid.value = IncomingMessage.contents
          break
      }
    })

    connection.addEventListener('error', (ErrorEvent) => {
      console.log('Error ->', ErrorEvent)

      checkReadyState()
    })

    connection.addEventListener('close', (CloseEvent) => {
      console.log('Closing ->', CloseEvent)

      checkReadyState()
    })

function closeConnection () {
  connection.close(1000, 'goodbye!')
}

function checkReadyState () {
  switch ( connection.readyState) {
    case 0:
      ready_state.value = 'CONNECTING'
      return ready_state
    case 1:
      ready_state.value = 'OPEN'
      return ready_state
    case 2:
      ready_state.value = 'CLOSING'
      return ready_state
    case 3:
      ready_state.value = 'CLOSED'
      return ready_state
    }
}

function receiveMessage (message: string) {
  const id = messages.length + 1

  messages.push({ id: id, text: message })

  return messages
}

function sendMessage () {
    const newMessage = document.getElementById('new_message') as HTMLInputElement

    console.log(newMessage.value)

    if (newMessage != null) {
      connection.send(newMessage.value)

    const clear = newMessage.value = ''

    return clear
  }
}

function scrollMessages () {
  const id = messages.length
  const element = document.getElementById('message' + id)

  if (element != null) {
    element.scrollIntoView({ behavior: 'smooth', block: 'end', inline: 'end' })
  }
}

watch(
  () => messages,
  () => {
    nextTick(() => {
        scrollMessages()
    })
  },
  { deep: true }
)
</script>

<template>
  <transition name="fade" appear>
    <div id="console" class="console">
      <div id="nav" class="nav">
        <span id="title" class="title">
          <span class="accent"> | </span>
          r e l a y
        </span>
          <span id="url" class="url">
            <span class="accent">|</span>
            server
            <span class="accent">.</span>
            {{ url }}
            <span class="accent">|</span>
            status
            <span class="accent">.</span>
            {{ ready_state }}
            <span class="accent">|</span>
          </span>
        <button class="close_connection_button" v-on:click="closeConnection()">
          [ close connection ]
        </button>
      </div>
      <hr class="hr">
      <div id="messages" class="messages">
        <p v-for="message in messages" :key="message" :id="'message' + message.id">
          {{ message.text }}
        </p>
      </div>
      <div id="base" class="base">
        <span id="uuid" class="uuid">
          <span class="accent">||</span>
          <span>uuid -</span>
          {{ uuid }}
        </span>
        <span id="connected_users" class="connected_users">
          <span class="accent">||</span>
          users -
          {{ connected_users }}
        </span>
        <input type="text" id="new_message" class="new_message" autofocus @keyup.enter="sendMessage()">
        <button class="new_message_button" v-on:click="sendMessage()">new message</button>
      </div>
    </div>
  </transition>
</template>

<style>
.fade-enter-from {
    opacity: 0;
  }
  .fade-enter-active {
    transition: opacity 2.5s ease;
  }
  .accent {
    color: darkslategray;
  }
  body {
    background: whitesmoke;
  }
  .hr {
    color: steelblue;
    grid-row-start: 2;
    width: 100%;
  }
  .console {
    background: gainsboro;
    border-radius: 5px;
    width: 85vw;
    height: 85vh;
    padding: 20px;
    margin: auto;
    box-shadow: 10px 10px 10px 0 rgba(white, white, white, 0.75);
    position: relative;
    display: grid;
    grid-template-rows: 30px 10px auto 50px;
    gap: 10px;
  }
  .console .nav {
    display: grid;
    grid-template-columns: 50% 35% auto;
    align-items: center;
    justify-items: stretch;
  }
  .nav .title {
    color: goldenrod;
    font: 20px Verdana, sans-serif;
    grid-column-start: 1;
  }
  .nav .url {
    color: goldenrod;
    font: 16px Verdana, sans-serif;
    grid-column-start: 2;

  }
  .nav .close_connection_button {
    background-color: goldenrod;
    border: 2px solid darkslategray;
    padding: 5px 10px;
    outline: none;
    color: black;
    font: 12px Verdana, sans-serif;
    transition: background-color 1s, color 1s, border 1s;
    grid-column-start: 3;
  }
  .nav .close_connection_button:hover {
    background-color: gainsboro;
    color: darkslategray;
    border: 2px solid goldenrod;
  }
  .console .messages {
    background-color: lightgray;
    border-radius: 8px;
    border: 2px solid lightslategray;
    border-left: 15px solid lightslategray;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: goldenrod lightslategray;
  }
  .console .base {
    background-color: goldenrod;
    border-radius: 8px;
    border: 2px solid darkslategray;
    display: grid;
    grid-row-start: 4;
    grid-template-columns: auto auto 45% 15%;
    align-items: center;
    justify-items: stretch;
  }
  .uuid {
    color: black;
    font: 14px Verdana, sans-serif;
    grid-column-start: 1;
  }
  .connected_users {
    color: black;
    font: 14px Verdana, sans-serif;
    grid-column-start: 2;
  }
  .new_message {
    height: 50%;
    width: 95%;
    background-color: lightgray;
    border: 2px solid darkslategray;
    color: black;
    font: 12px Courier, monospace;
    outline: none;
    grid-column-start: 3;
  }
  .new_message_button {
    background-color: goldenrod;
    border: 2px solid darkslategray;
    border-radius: 100px;
    height: 75%;
    width: 95%;
    color: black;
    font: 12px Verdana, sans-serif;
    transition: background-color 1s, color 1s, border 1s;
    grid-column-start: 4;
  }
  .new_message_button:hover {
    background-color: darkslategray;
    color: goldenrod;
  }
</style>
