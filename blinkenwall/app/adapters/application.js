import Adapter from '@ember-data/adapter';
import { run } from '@ember/runloop';
import { Promise } from 'rsvp';
import { inject as service } from '@ember/service';
import jQuery from 'jquery';

export default Adapter.extend({
  serverConnection: service(),

  createRecord(store /*jshint unused:false*/, type, snapshot) {
    var data = this.serialize(snapshot, { includeId: false });
    switch (type.modelName) {
      case 'shader-content':
        return new Promise((resolve, reject) => {
          this.serverConnection.send(
            jQuery.extend(
              {
                cmd: 'shader create',
              },
              data
            ),
            (response) => {
              if (response.id) {
                data.id = response.id;
                data.commit = response.commit;
                run(null, resolve, data);
              } else {
                run(null, reject, data);
              }
            }
          );
        });
      case 'shader':
        return new Promise((resolve, reject) => {
          if (data.content) {
            run(null, resolve, {
              id: data.content,
              content: data.content,
            });
          } else {
            run(null, reject, 'Cannot create a shader without content');
          }
        });
    }
  },

  deleteRecord(
    store /*jshint unused:false*/,
    type /*jshint unused:false*/,
    snapshot
  ) {
    return new Promise((resolve, reject) => {
      run(null, reject, 'Not implemented yet: ' + snapshot);
    });
  },

  findAll(store /*jshint unused:false*/, type) {
    if (type.modelName !== 'shader') {
      return null;
    }
    return new Promise((resolve, reject) => {
      this.serverConnection.send(
        {
          cmd: 'shader list',
        },
        (response) => {
          if (response.ids) {
            run(
              null,
              resolve,
              response.ids.map((id) => {
                return { id: id, type: 'shader', content: id };
              })
            );
          } else {
            run(null, reject, response);
          }
        }
      );
    });
  },

  findRecord(store /*jshint unused:false*/, type, id) {
    switch (type.modelName) {
      case 'shader':
        return new Promise((resolve) => {
          run(null, resolve, {
            id: id,
            content: id,
          });
        });
      case 'shader-content':
        return new Promise((resolve, reject) => {
          this.serverConnection.send(
            {
              cmd: 'shader read',
              id: id,
            },
            (response) => {
              if (response.source) {
                run(null, resolve, {
                  id: id,
                  title: response.title,
                  description: response.description,
                  source: response.source,
                  commit: response.commit,
                });
              } else {
                run(null, reject, response);
              }
            }
          );
        });
    }
  },

  query(store, type, query) {
    // jshint unused:false
    // not implemented yet on the server!
    return new Promise((resolve, reject) => {
      run(null, reject, 'Not implemented yet');
    });
  },

  updateRecord(store, type, snapshot) {
    // jshint unused:false
    var data = this.serialize(snapshot, { includeId: true });
    switch (type.modelName) {
      case 'shader-content':
        return new Promise((resolve, reject) => {
          this.serverConnection.send(
            jQuery.extend(
              {
                cmd: 'shader write',
              },
              data
            ),
            (response) => {
              if (response.id) {
                run(null, resolve, data);
              } else {
                run(null, reject, data);
              }
            }
          );
        });
      case 'shader':
        return new Promise((resolve, reject) => {
          if (data.content) {
            run(null, resolve, {
              id: data.content,
              content: data.content,
            });
          } else {
            run(null, reject, 'Cannot create a shader without content');
          }
        });
    }
  },
});
