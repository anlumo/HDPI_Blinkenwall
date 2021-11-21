import Model, { attr } from '@ember-data/model';

export default Model.extend({
  title: attr('string'),
  description: attr('string'),
  source: attr('string'),
  commit: attr('string'),
});
