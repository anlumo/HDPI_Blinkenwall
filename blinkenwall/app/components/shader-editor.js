import { inject as service } from '@ember/service';
import Component from '@ember/component';

export default Component.extend({
  store: service(),
  editorSource: "",
  shader: null,

  didInsertElement() {
    let shader = this.shader;
    let source = shader.get('source');
    this.set('editorSource', source);
    this.sendAction('compile', source);
  },

  actions: {
    valueUpdated(newValue) {
      this.set('editorSource', newValue);
      this.shader.set('source', this.editorSource);
    },
    compile() {
      this.shader.set('source', this.editorSource);
      this.sendAction('compile', this.get('shader.source'));
    },
    maximize() {

    },
    help() {

    }
  }
});
