import Ember from 'ember';

export default Ember.Controller.extend({
  toxid: "",
  serverConnection: Ember.inject.service(),

  init() {
    this._super(...arguments);

    fetch('/tox/toxid.txt').then(response => response.text()).then(data => {
      this.set('toxid', data);
    });
  },

  actions: {
    start() {
      this.get('serverConnection').send({
        cmd: "tox start",
      });
    },
  },
});
