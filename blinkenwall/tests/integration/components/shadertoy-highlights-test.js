import { moduleForComponent, test } from 'ember-qunit';
import hbs from 'htmlbars-inline-precompile';

moduleForComponent('shadertoy-highlights', 'Integration | Component | shadertoy highlights', {
  integration: true
});

test('it renders', function(assert) {

  // Set any properties with this.set('myProperty', 'value');
  // Handle any actions with this.on('myAction', function(val) { ... });

  this.render(hbs`{{shadertoy-highlights}}`);

  assert.equal(this.$().text().trim(), '');

  // Template block usage:
  this.render(hbs`
    {{#shadertoy-highlights}}
      template block text
    {{/shadertoy-highlights}}
  `);

  assert.equal(this.$().text().trim(), 'template block text');
});
