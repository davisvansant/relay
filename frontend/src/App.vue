<template>
  <transition name="fade" appear>
    <div id="console" class="console">
      <div id="nav" class="nav">
        <span id="title" class="title">
          <span> | </span>
          r e l a y
        </span>
          <span id="url" class="url">
            <span>|</span>
            server
            <span>.</span>
            {{ url }}
            <span>|</span>
            status
            <span>.</span>
            {{ ready_state }}
            <span>|</span>
          </span>
        <button class="close_connection_button">
          [ close connection ]
        </button>
      </div>
      <hr class="hr">
      <div id="messages" class="messages">
        <p id="message" v-for="message in received_messages" :key="message">
          {{ message }}
        </p>
      </div>
      <div id="base" class="base">
        <span id="uuid" class="uuid">
          ||
          uuid -
          {{ uuid }}
        </span>
        <span id="connected_users" class="connected_users">
          users -
          {{ connected_users }}
        </span>
        <input type="text" id="new_message" class="new_message" autofocus>
        <button class="new_message_button">new message</button>
      </div>
    </div>
  </transition>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  name: 'app',
  data () {
    return {
      url: '',
      ready_state: '',
      received_messages: [],
      uuid: '',
      connected_users: ''
    }
  }
})
</script>

<style>
  .fade-enter-from {
    opacity: 0;
  }
  .fade-enter-active {
    transition: opacity 2.5s ease;
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
