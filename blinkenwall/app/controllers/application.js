import $ from 'jquery';
import { inject as service } from '@ember/service';
import Controller from '@ember/controller';

export default Controller.extend({
  serverConnection: service(),
  actions: {
    showSidebar(sidebar) {
      $(`#${sidebar}`).sidebar('show');
    },
    hideSidebar(sidebar) {
      $(`#${sidebar}`).sidebar('hide');
    },
    stop() {
      this.serverConnection.send({
        cmd: 'turnoff',
      });
    },
  },
});
