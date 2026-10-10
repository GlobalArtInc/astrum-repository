
/*global Ext, AS*/

/**
 * Repository "Settings" panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.RepositorySettings', {
  extend: 'AS.view.SettingsPanel',
  alias: 'widget.as-coreui-repository-settings',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {


      title: AS.I18n.get('Repository_RepositorySettings_Title'),

      settingsForm: { xtype: 'as-coreui-repository-settings-form' },

      dockedItems: null
    });

    this.callParent();
  }
});
