import Ember from 'ember';

export default Ember.Service.extend({
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
    let index = "" + this.get('index');
    if(cb) {
      this.callbacks[index] = cb;
    }
    let msg = Ember.$.extend({ id: index }, message);
    let ws = this.get('ws');
    if(ws.readyState === 1) {
      console.log("Sending websocket message", msg);
      this.get('ws').send(JSON.stringify(msg));
    } else {
      this.get('messageQueue').push(msg);
    }
    this.incrementProperty('index');
  },

  onOpen() {
    console.log("Websocket connection opened.");
    let ws = this.get('ws');
    this.get('messageQueue').forEach((msg) => {
      ws.send(JSON.stringify(msg));
    });
    this.set('messageQueue', []);
  },

  onMessage(event) {
    let msg = JSON.parse(event.data);
    console.log("Received websocket message", msg);
    let id = msg.id;
    if(id) {
      let cb = this.callbacks[id];
      if(cb) {
        cb(msg);
        delete this.callbacks[id];
      }
    }
  },

  onClose() {
    console.log("Websocket closed, trying to reconnect in 2s...");
    Ember.run.later(this, "reconnect", 2000);
  },

  reconnect() {
    console.log("Connecting to websocket...");
    let ws = new WebSocket(`ws://${document.location.hostname}:${this.get('port')}/blinkenwall`);
    ws.onopen = this.onOpen.bind(this);
    ws.onmessage = this.onMessage.bind(this);
    ws.onerror = this.onClose.bind(this);
    ws.onclose = this.onClose.bind(this);
    this.set('ws', ws);
  },
});
