import { Promise } from 'rsvp';
import { inject as service } from '@ember/service';
import Component from '@ember/component';

export default Component.extend({
  classNames: ['ui', 'segment'],
  store: service(),

  shaders: [],

  didInsertElement() {
    this._super(...arguments);
    this.store.findAll('shader').then((shaders) => {
      Promise.all(
        shaders.map((s) => {
          return s.get('content');
        })
      ).then(() => {
        let list = shaders.toArray().sort((a, b) => {
          let nameA = a.get('content.title'),
            nameB = b.get('content.title');
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
  },
});
