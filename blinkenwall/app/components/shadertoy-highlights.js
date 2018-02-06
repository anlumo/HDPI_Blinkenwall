import Ember from 'ember';

export default Ember.Component.extend({
  classNames: ['ui', 'segment'],
  store: Ember.inject.service(),

  count: 5,
  highlights: [],

  shuffle(array) {
    let top = array.length;
    if(top) {
      while(--top) {
        let current = Math.floor(Math.random() * (top + 1));
        [array[current], array[top]] = [array[top], array[current]];
      }
    }
    return array;
  },

  didInsertElement() {
    this.get('store').findAll('shader').then((shaders) => {
      let list = shaders.toArray();
      this.shuffle(list);
      this.set('highlights', list.slice(0, this.get('count')));
    });
  }
});
