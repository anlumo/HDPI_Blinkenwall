import Ember from 'ember';

export default Ember.Controller.extend({
  serverConnection: Ember.inject.service(),

  isNew: Ember.computed('model.id', function() {
    return this.get('model.id') == null;
  }),

  actions: {
    compile(source) {
      Ember.run.once(() => {
        this.set('source', source);
      });
    },
    publish() {
      this.get('model').save();
    },
    activate() {
      this.get('serverConnection').send({
        cmd: "shader activate",
        id: this.get('model.id')
      });
    }
  }
});
