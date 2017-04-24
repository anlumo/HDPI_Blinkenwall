import Ember from 'ember';

export default Ember.Controller.extend({
  actions: {
    compile(source) {
      Ember.run.once(() => {
        this.set('source', source);
      });
    }
  }
});
