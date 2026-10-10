
/*global Ext, AS*/

/**
 * Configuration for the repository routing rule.
 *
 * @since 3.16
 */
Ext.define('AS.coreui.view.repository.facet.RoutingRuleFacetViewController', {
  extend: 'Ext.app.ViewController',
  alias: 'controller.routingRuleViewController',
  requires: [
    'AS.State',
    'AS.I18n'
  ],

  control: {
    '#': {
      beforeRender: 'onBeforeRender',
      afterRender: 'onAfterRender'
    }
  },

  onBeforeRender: function() {
    this.lookupReference('routingRuleCombo').setHelpText(AS.I18n.get('Repository_Facet_RoutingRuleFacet_HelpText'));
    this.getViewModel().set('title', AS.I18n.get('Repository_Facet_RoutingRuleFacet_Title'));
  },

  onAfterRender: function() {
    var combo = this.lookupReference('routingRuleCombo'),
        store = this.getStore('RoutingRules'),
        model = store.getModel();

    store.load(function() {
      store.insert(0, model.create({
        id: '',
        name: 'None'
      }));

      if (store.find('id', combo.getValue()) === -1) {
        combo.setValue('');
      }
    });
  }
});
