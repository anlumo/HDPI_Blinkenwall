import Ember from 'ember';

export default Ember.Controller.extend({
  text: "",
  serverConnection: Ember.inject.service(),

  actions: {
    showText() {
      this.get('serverConnection').send({
        cmd: "show poetry",
        text: this.get('text'),
      });
    },
  }
});
