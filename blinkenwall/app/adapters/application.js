import Ember from 'ember';
import DS from 'ember-data';

export default DS.Adapter.extend({
  serverConnection: Ember.inject.service(),

  createRecord(store /*jshint unused:false*/, type, snapshot) {
    var data = this.serialize(snapshot, {includeId: false});
    switch(type.modelName) {
      case "shader-content":
        return new Ember.RSVP.Promise((resolve, reject) => {
          this.get('serverConnection').send(jQuery.extend({
            cmd: "shader create",
          }, data), (response) => {
            if(response.id) {
              data.id = response.id;
              Ember.run(null, resolve, data);
            } else {
              Ember.run(null, reject, data);
            }
          });
        });
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
        cmd: "shader list"
      }, (response) => {
        if(response.ids) {
          Ember.run(null, resolve, response.ids.map((id) => { return {id: id, type: "shader", content: id}; }));
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
            cmd: "shader read",
            id: id
          }, (response) => {
            if(response.source) {
              Ember.run(null, resolve, {
                id: id,
                title: response.title,
                description: response.description,
                source: response.source
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
    var data = this.serialize(snapshot, {includeId: true});
    switch(type.modelName) {
      case "shader-content":
        return new Ember.RSVP.Promise((resolve, reject) => {
          this.get('serverConnection').send(jQuery.extend({
            cmd: "shader write"
          }, data), (response) => {
            if(response.id) {
              Ember.run(null, resolve, data);
            } else {
              Ember.run(null, reject, data);
            }
          });
        });
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
});
