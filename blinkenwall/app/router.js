import EmberRouter from '@ember/routing/router';
import config from 'blinkenwall/config/environment';

export default class Router extends EmberRouter {
  location = config.locationType;
  rootURL = config.rootURL;
}

Router.map(function() {
  this.route('shadertoy', function() {
    this.route('edit', {
      path: '/edit/:id'
    });
    this.route('all');
  });
  this.route('youtube');
  this.route('emulator');
  this.route('vnc');
  this.route('retro');
  this.route('tox');
  this.route('poetry');
});
