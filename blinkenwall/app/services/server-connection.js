import { later } from '@ember/runloop';
import $ from 'jquery';
import Service from '@ember/service';

export default Service.extend({
  port: 1337,
  ws: null,
  index: 1,
  messageQueue: [],
  callbacks: {},

  init() {
    this._super(...arguments);
    this.reconnect();
  },

  send(message, cb) {
    let index = '' + this.index;
    if (cb) {
      this.callbacks[index] = cb;
    }
    let msg = $.extend({ req: index }, message);
    let ws = this.ws;
    if (ws && ws.readyState === 1) {
      console.log('Sending websocket message', msg);
      this.ws.send(JSON.stringify(msg));
    } else {
      this.messageQueue.push(msg);
    }
    this.incrementProperty('index');
  },

  onOpen() {
    console.log('Websocket connection opened.');
    let ws = this.ws;
    this.messageQueue.forEach((msg) => {
      ws.send(JSON.stringify(msg));
    });
    this.set('messageQueue', []);
  },

  onMessage(event) {
    let msg = JSON.parse(event.data);
    console.log('Received websocket message', msg);
    let req = msg.req;
    if (req) {
      let cb = this.callbacks[req];
      if (cb) {
        cb(msg);
        delete this.callbacks[req];
      }
    }
  },

  onClose() {
    if (this.ws) {
      console.log('Websocket closed, trying to reconnect in 2s...');
      this.set('ws', null);
      later(this, 'reconnect', 2000);
    }
  },

  reconnect() {
    console.log('Connecting to websocket...');
    let ws = new WebSocket(
      `${document.location.protocol === 'https:'?'wss':'ws'}://${document.location.hostname}/blinkenwall`
    );
    ws.onopen = this.onOpen.bind(this);
    ws.onmessage = this.onMessage.bind(this);
    ws.onerror = this.onClose.bind(this);
    ws.onclose = this.onClose.bind(this);
    this.set('ws', ws);
  },
});
