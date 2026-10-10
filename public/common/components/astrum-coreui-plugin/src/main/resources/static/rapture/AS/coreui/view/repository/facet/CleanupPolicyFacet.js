
/*global Ext, AS*/

/**
 * Configuration for repository cleanup policy facet.
 *
 * @since 3.14
 */
Ext.define('AS.coreui.view.repository.facet.CleanupPolicyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-cleanup-policy-facet',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_CleanupPolicyFacet_Title'),

        items: {
          xtype: 'as-itemselector',
          name: 'attributes.cleanup.policyName',
          itemId: 'cleanupPolicyName',
          fieldLabel: AS.I18n.get('Repository_Facet_CleanupPolicyFacet_Policy_FieldLabel'),
          helpText: AS.I18n.get('Repository_Facet_CleanupPolicyFacet_Policy_HelpText'),
          buttons: ['add', 'remove'],
          fromTitle: AS.I18n.get('Repository_Facet_CleanupPolicyFacet_Policy_FromTitle'),
          toTitle: AS.I18n.get('Repository_Facet_CleanupPolicyFacet_Policy_ToTitle'),
          editable: false,
          store: 'CleanupPolicy',
          valueField: 'name',
          displayField: 'name',
          allowBlank: true,
          delimiter: null,
          forceSelection: true,
          queryMode: 'local',
          triggerAction: 'all',
          selectOnFocus: false
        }
      }
    ];

    me.callParent();
  }
});
