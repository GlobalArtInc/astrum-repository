
/*global Ext, NX*/

/**
 * Repository "Settings" panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.RepositorySettings', {
  extend: 'NX.view.SettingsPanel',
  alias: 'widget.nx-coreui-repository-settings',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {


      title: NX.I18n.get('Repository_RepositorySettings_Title'),

      settingsForm: { xtype: 'nx-coreui-repository-settings-form' },

      dockedItems: null
    });

    this.callParent();
  }
});
