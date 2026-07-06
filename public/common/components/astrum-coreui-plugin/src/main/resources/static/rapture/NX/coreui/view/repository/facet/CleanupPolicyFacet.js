
/*global Ext, NX*/

/**
 * Configuration for repository cleanup policy facet.
 *
 * @since 3.14
 */
Ext.define('NX.coreui.view.repository.facet.CleanupPolicyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-cleanup-policy-facet',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_CleanupPolicyFacet_Title'),

        items: {
          xtype: 'nx-itemselector',
          name: 'attributes.cleanup.policyName',
          itemId: 'cleanupPolicyName',
          fieldLabel: NX.I18n.get('Repository_Facet_CleanupPolicyFacet_Policy_FieldLabel'),
          helpText: NX.I18n.get('Repository_Facet_CleanupPolicyFacet_Policy_HelpText'),
          buttons: ['add', 'remove'],
          fromTitle: NX.I18n.get('Repository_Facet_CleanupPolicyFacet_Policy_FromTitle'),
          toTitle: NX.I18n.get('Repository_Facet_CleanupPolicyFacet_Policy_ToTitle'),
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
