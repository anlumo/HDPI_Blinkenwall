import { inject as service } from '@ember/service';
import Route from '@ember/routing/route';

export default Route.extend({
  defaultSource: "void mainImage( out vec4 fragColor, in vec2 fragCoord ) {\n\tfragColor = vec4(0.0, 0.0, 1.0, 1.0);\n}",

  store: service(),
  model: function(shaderId) {
    if(shaderId.id === "new") {
      return this.store.createRecord('shader-content', {source: this.defaultSource});
    }
    return this.store.findRecord('shader-content', shaderId.id);
  },
});
