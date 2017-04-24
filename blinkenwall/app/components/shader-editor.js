import Ember from 'ember';

export default Ember.Component.extend({
  store: Ember.inject.service(),
  defaultSource: "void mainImage( out vec4 fragColor, in vec2 fragCoord ) {\n\tfragColor = vec4(0.0, 0.0, 1.0, 1.0);\n}",
  source: "",
  editorSource: "",

  shaderId: "new",

  updateSource: Ember.observer('shaderId', function() {
    let id = this.get('shaderId').id;
    if(id === "new") {
      this.set('editorSource', this.get('defaultSource'));
      this.set('source', this.get('defaultSource'));
      this.sendAction('compile', this.get('source'));
    } else {
      this.get('store').findRecord('shader-content', id).then((shader) => {
        let source = shader.get('source');
        this.set('editorSource', source);
        this.set('source', source);
        this.sendAction('compile', source);
      });
    }
  }),

  didInsertElement() {
    this.updateSource();
  },

  actions: {
    valueUpdated(newValue) {
      this.set('source', newValue);
    },
    compile() {
      this.sendAction('compile', this.get('source'));
    },
    maximize() {

    },
    help() {

    }
  }
});
