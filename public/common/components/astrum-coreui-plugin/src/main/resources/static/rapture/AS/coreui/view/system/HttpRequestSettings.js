
/*global Ext, AS*/

/**
 * Http request settings fields.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.system.HttpRequestSettings', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-system-httprequestsettings',
  requires: [
    'AS.I18n'
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
          fieldLabel: AS.I18n.get('System_HttpRequestSettings_UserAgentCustomization_FieldLabel'),
          helpText: AS.I18n.get('System_HttpRequestSettings_UserAgentCustomization_HelpText')
        },
        {
          xtype: 'numberfield',
          name: 'timeout',
          fieldLabel: AS.I18n.get('System_HttpRequestSettings_Timeout_FieldLabel'),
          helpText: AS.I18n.get('System_HttpRequestSettings_Timeout_HelpText'),
          allowDecimals: false,
          allowExponential: false,
          minValue: 1,
          maxValue: 3600,
          emptyText: '20'
        },
        {
          xtype: 'numberfield',
          name: 'retries',
          fieldLabel: AS.I18n.get('System_HttpRequestSettings_Attempts_FieldLabel'),
          helpText: AS.I18n.get('System_HttpRequestSettings_Attempts_HelpText'),
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
