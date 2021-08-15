import Model, { belongsTo } from '@ember-data/model';

export default Model.extend({
  content: belongsTo('shader-content')
});
