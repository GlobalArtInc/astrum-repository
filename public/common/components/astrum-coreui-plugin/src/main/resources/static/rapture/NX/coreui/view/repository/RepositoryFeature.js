
/*global Ext, NX*/

/**
 * Repository feature panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.RepositoryFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-repository-feature',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'repository-default',

      masters: [
        { xtype: 'nx-coreui-repository-list' }
      ],

      tabs: { xtype: 'nx-coreui-repository-settings' },

      nxActions: [
        { xtype: 'button', text: NX.I18n.get('Repository_RepositoryFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true },
        { xtype: 'button', text: NX.I18n.get('Repository_RepositoryFeature_RebuildIndex_Button'), iconCls: 'x-fa fa-wrench', action: 'rebuildIndex', disabled: true },
        { xtype: 'button', text: NX.I18n.get('Repository_RepositoryFeature_InvalidateCache_Button'), iconCls: 'x-fa fa-eraser', action: 'invalidateCache', disabled: true },
        { xtype: 'button', text: NX.I18n.get('Repository_RepositoryFeature_HealthCheckEnable_Button'), iconCls: 'x-fa fa-chart-line', action: 'toggleHealthCheck', disabled: true }
      ]
    });

    this.callParent();
  }
});
