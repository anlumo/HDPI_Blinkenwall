import { moduleForComponent, test } from 'ember-qunit';
import hbs from 'htmlbars-inline-precompile';

moduleForComponent(
  'shadertoy-preview',
  'Integration | Component | shadertoy preview',
  {
    integration: true,
  }
);

test('it renders', function (assert) {
  // Set any properties with this.set('myProperty', 'value');
  // Handle any actions with this.on('myAction', function(val) { ... });

  this.render(hbs`{{shadertoy-preview}}`);

  assert.equal(this.$().text().trim(), '');

  // Template block usage:
  this.render(hbs`
    {{#shadertoy-preview}}
      template block text
    {{/shadertoy-preview}}
  `);

  assert.equal(this.$().text().trim(), 'template block text');
});
