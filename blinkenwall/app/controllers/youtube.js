import Ember from 'ember';

export default Ember.Controller.extend({
  url: "",
  serverConnection: Ember.inject.service(),

  actions: {
    playVideo() {
      console.log("Play video", this.get('url'));
      this.get('serverConnection').send({
        cmd: "video play",
        url: this.get('url'),
      });
    },
    stopVideo() {
      this.get('serverConnection').send({
        cmd: "turnoff"
      });
    }
  }
});
