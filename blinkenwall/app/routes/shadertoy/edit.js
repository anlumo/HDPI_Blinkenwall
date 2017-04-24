import Ember from 'ember';

export default Ember.Route.extend({
  defaultSource: "void mainImage( out vec4 fragColor, in vec2 fragCoord ) {\n\tfragColor = vec4(0.0, 0.0, 1.0, 1.0);\n}",

  store: Ember.inject.service(),
  model: function(shaderId) {
    if(shaderId.id === "new") {
      return this.get('store').createRecord('shader-content', {source: this.get('defaultSource')});
    }
    return this.get('store').findRecord('shader-content', shaderId.id);
  },
});
