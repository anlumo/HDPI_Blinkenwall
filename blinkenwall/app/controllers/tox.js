import { inject as service } from '@ember/service';
import Controller from '@ember/controller';

export default Controller.extend({
  toxid: "",
  serverConnection: service(),

  init() {
    this._super(...arguments);

    fetch('/tox/toxid.txt').then(response => response.text()).then(data => {
      this.set('toxid', data);
    });
  },

  actions: {
    start() {
      this.serverConnection.send({
        cmd: "tox start",
      });
    },
  },
});
