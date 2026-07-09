
/*global Ext, NX*/

/**
 * Http request settings fields.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.system.HttpRequestSettings', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-system-httprequestsettings',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      items: [
        {
          xtype: 'textfield',
          name: 'userAgentSuffix',
          fieldLabel: NX.I18n.get('System_HttpRequestSettings_UserAgentCustomization_FieldLabel'),
          helpText: NX.I18n.get('System_HttpRequestSettings_UserAgentCustomization_HelpText')
        },
        {
          xtype: 'numberfield',
          name: 'timeout',
          fieldLabel: NX.I18n.get('System_HttpRequestSettings_Timeout_FieldLabel'),
          helpText: NX.I18n.get('System_HttpRequestSettings_Timeout_HelpText'),
          allowDecimals: false,
          allowExponential: false,
          minValue: 1,
          maxValue: 3600,
          emptyText: '20'
        },
        {
          xtype: 'numberfield',
          name: 'retries',
          fieldLabel: NX.I18n.get('System_HttpRequestSettings_Attempts_FieldLabel'),
          helpText: NX.I18n.get('System_HttpRequestSettings_Attempts_HelpText'),
          allowDecimals: false,
          allowExponential: false,
          minValue: 0,
          maxValue: 10,
          emptyText: '2'
        }
      ]
    });

    this.callParent();
  }

});
