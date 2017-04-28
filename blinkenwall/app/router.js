import Ember from 'ember';
import config from './config/environment';

const Router = Ember.Router.extend({
  location: config.locationType,
  rootURL: config.rootURL
});

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
});

export default Router;
