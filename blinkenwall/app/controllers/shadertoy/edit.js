import { once } from '@ember/runloop';
import { computed } from '@ember/object';
import { inject as service } from '@ember/service';
import Controller from '@ember/controller';

export default Controller.extend({
  serverConnection: service(),

  isNew: computed('model.id', function() {
    return this.get('model.id') == null;
  }),

  actions: {
    compile(source) {
      once(() => {
        this.set('source', source);
      });
    },
    publish() {
      this.model.save();
    },
    activate() {
      this.serverConnection.send({
        cmd: "shader activate",
        id: this.get('model.id')
      });
    }
  }
});
