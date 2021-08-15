import { inject as service } from '@ember/service';
import Controller from '@ember/controller';

export default Controller.extend({
  text: "",
  serverConnection: service(),

  actions: {
    showText() {
      this.serverConnection.send({
        cmd: "show poetry",
        text: this.text,
      });
    },
  }
});
