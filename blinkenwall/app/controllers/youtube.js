import { inject as service } from '@ember/service';
import Controller from '@ember/controller';

export default Controller.extend({
  url: '',
  serverConnection: service(),

  actions: {
    playVideo() {
      console.log('Play video', this.url);
      this.serverConnection.send({
        cmd: 'video play',
        url: this.url,
      });
    },
    stopVideo() {
      this.serverConnection.send({
        cmd: 'turnoff',
      });
    },
  },
});
