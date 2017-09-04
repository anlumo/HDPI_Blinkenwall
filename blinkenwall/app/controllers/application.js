import Ember from 'ember';

export default Ember.Controller.extend({
  serverConnection: Ember.inject.service(),
  actions: {
    showSidebar(sidebar) {
      Ember.$(`#${sidebar}`).sidebar('show');
    },
    hideSidebar(sidebar) {
      Ember.$(`#${sidebar}`).sidebar('hide');
    },
    stop() {
      this.get('serverConnection').send({
        cmd: "turnoff"
      });
    },
  }
});
