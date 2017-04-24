import Ember from 'ember';
import DS from 'ember-data';

export default DS.Adapter.extend({
  serverConnection: Ember.inject.service(),

  createRecord(store /*jshint unused:false*/, type, snapshot) {
    var data = this.serialize(snapshot, {includeId: false});
    switch(type.modelName) {
      case "shader-content":
        return new Ember.RSVP.Promise((resolve, reject) => {
          this.get('serverConnection').send({
            cmd: "create",
            content: JSON.stringify(data)
          }, (response) => {
            if(response.key) {
              data.id = response.key;
              Ember.run(null, resolve, data);
            } else {
              Ember.run(null, reject, data);
            }
          });
        });
        break;
      case "shader":
        return new Ember.RSVP.Promise((resolve, reject) => {
          if(data.content) {
            Ember.run(null, resolve, {
              id: data.content,
              content: data.content
            });
          } else {
            Ember.run(null, reject, "Cannot create a shader without content");
          }
        });
    }
  },

  deleteRecord(store /*jshint unused:false*/, type /*jshint unused:false*/, snapshot) {
    // not implemented yet on the server!
    return new Ember.RSVP.Promise((resolve, reject) => {
      Ember.run(null, reject, "Not implemented yet: " + snapshot);
    });
  },

  findAll(store /*jshint unused:false*/, type) {
    if(type.modelName !== "shader") {
      return null;
    }
    return new Ember.RSVP.Promise((resolve, reject) => {
      this.get('serverConnection').send({
        cmd: "list"
      }, (response) => {
        if(response.keys) {
          Ember.run(null, resolve, response.keys.map((key) => { return {id: key, type: "shader", content: key}; }));
        } else {
          Ember.run(null, reject, response);
        }
      });
    });
  },

  findRecord(store /*jshint unused:false*/, type, id) {
    switch(type.modelName) {
      case "shader":
        return new Ember.RSVP.Promise((resolve) => {
          Ember.run(null, resolve, {
            id: id,
            content: id
          });
        });
      case "shader-content":
        return new Ember.RSVP.Promise((resolve, reject) => {
          this.get('serverConnection').send({
            cmd: "read",
            key: id
          }, (response) => {
            if(response.data) {
              let shader = JSON.parse(response.data);
              Ember.run(null, resolve, {
                id: id,
                title: shader.title,
                description: shader.description,
                source: shader.source
              });
            } else {
              Ember.run(null, reject, response);
            }
          });
        });
    }
  },

  query(store, type, query) { // jshint unused:false
    // not implemented yet on the server!
    return new Ember.RSVP.Promise((resolve, reject) => {
      Ember.run(null, reject, "Not implemented yet");
    });
  },

  updateRecord(store, type, snapshot) { // jshint unused:false
    // not implemented yet on the server!
    return new Ember.RSVP.Promise((resolve, reject) => {
      Ember.run(null, reject, "Not implemented yet");
    });
  },
});
