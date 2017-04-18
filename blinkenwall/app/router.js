import Ember from 'ember';
import config from './config/environment';

const Router = Ember.Router.extend({
  location: config.locationType,
  rootURL: config.rootURL
});

Router.map(function() {
  this.route('shadertoy', function() {
    this.route('edit');
  });
  this.route('youtube');
  this.route('emulator');
  this.route('vnc');
});

export default Router;
