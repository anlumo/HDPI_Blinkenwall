import Ember from 'ember';

export default Ember.Component.extend({
  store: Ember.inject.service(),
  editorSource: "",
  shader: null,

  didInsertElement() {
    let shader = this.get('shader');
    let source = shader.get('source');
    this.set('editorSource', source);
    this.sendAction('compile', source);
  },

  actions: {
    valueUpdated(newValue) {
      this.set('editorSource', newValue);
      this.get('shader').set('source', this.get('editorSource'));
    },
    compile() {
      this.get('shader').set('source', this.get('editorSource'));
      this.sendAction('compile', this.get('shader.source'));
    },
    maximize() {

    },
    help() {

    }
  }
});
