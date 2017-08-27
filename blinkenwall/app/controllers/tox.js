import Ember from 'ember';

export default Ember.Controller.extend({
  toxid: "",
  serverConnection: Ember.inject.service(),

  init() {
    this._super(...arguments);

    $.get('/tox/toxid.txt', data => {
      this.set('toxid', data);
    }, "text");
  },

  actions: {
    start() {
      this.get('serverConnection').send({
        cmd: "tox start",
      });
    },
  },
});
