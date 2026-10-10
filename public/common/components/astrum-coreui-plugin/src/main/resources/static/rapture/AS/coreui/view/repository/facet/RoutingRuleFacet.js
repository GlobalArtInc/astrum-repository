
/*global Ext, AS*/

/**
 * Configuration for the repository routing rule.
 *
 * @since 3.16
 */
Ext.define('AS.coreui.view.repository.facet.RoutingRuleFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-routing-rule-facet',
  requires: [
    'AS.I18n',
    'AS.coreui.model.RoutingRule',
    'AS.coreui.view.repository.facet.RoutingRuleFacetViewController'
  ],

  controller: 'routingRuleViewController',

  viewModel: {
    stores: {
      RoutingRules: {
        extend: 'Ext.data.Store',
        model: 'AS.coreui.model.RoutingRule',
        proxy: {
          type: 'rest',
          url: 'service/rest/internal/ui/routing-rules'
        },
        sorters:[{
          sorterFn: function(record1, record2) {
            var name1 = record1.get('name'),
                name2 = record2.get('name');

            if ('None' === name1) {
              return -1;
            } else if ('None' === name2) {
              return 1;
            }

            return name1.localeCompare(name2);
          }
        }]
      }
    }
  },

  items: [
    {
      xtype: 'fieldset',
      cls: 'as-form-section',
      bind: {
        title: '{title}'
      },

      items: [
        {
          xtype: 'combo',
          reference: 'routingRuleCombo',
          name: 'routingRuleId',
          displayField: 'name',
          valueField: 'id',
          bind: {
            store: '{RoutingRules}'
          },
          queryMode: 'local',
          typeAhead: true,
          forceSelection: true
        }
      ]
    }
  ]
});
