import Ember from 'ember';

export default Ember.Component.extend({
  classNames: ['ui', 'segment'],
  store: Ember.inject.service(),

  shaders: [],

  didInsertElement() {
    this.get('store').findAll('shader').then((shaders) => {
      Ember.RSVP.Promise.all(shaders.map((s) => { return s.get('content'); } )).then(() => {
        let list = shaders.toArray().sort((a, b) => {
          let nameA = a.get('content.title'), nameB = b.get('content.title');
          console.log(`a = ${nameA}, b = ${nameB}`);
          if (nameA < nameB) {
            return -1;
          }
          if (nameA > nameB) {
            return 1;
          }

          // names must be equal
          return 0;
        });
        this.set('shaders', list);
      });
    });
  }
});
