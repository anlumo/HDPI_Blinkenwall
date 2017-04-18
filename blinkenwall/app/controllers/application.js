import Ember from 'ember';

export default Ember.Controller.extend({
  actions: {
    showSidebar(sidebar) {
      Ember.$(`#${sidebar}`).sidebar('show');
    },
    hideSidebar(sidebar) {
      Ember.$(`#${sidebar}`).sidebar('hide');
    }
  }
});
