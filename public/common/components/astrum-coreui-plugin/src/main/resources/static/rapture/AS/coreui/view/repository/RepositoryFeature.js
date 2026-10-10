
/*global Ext, AS*/

/**
 * Repository feature panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.RepositoryFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-repository-feature',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'repository-default',

      masters: [
        { xtype: 'as-coreui-repository-list' }
      ],

      tabs: { xtype: 'as-coreui-repository-settings' },

      asActions: [
        { xtype: 'button', text: AS.I18n.get('Repository_RepositoryFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true },
        { xtype: 'button', text: AS.I18n.get('Repository_RepositoryFeature_RebuildIndex_Button'), iconCls: 'x-fa fa-wrench', action: 'rebuildIndex', disabled: true },
        { xtype: 'button', text: AS.I18n.get('Repository_RepositoryFeature_InvalidateCache_Button'), iconCls: 'x-fa fa-eraser', action: 'invalidateCache', disabled: true },
        { xtype: 'button', text: AS.I18n.get('Repository_RepositoryFeature_HealthCheckEnable_Button'), iconCls: 'x-fa fa-chart-line', action: 'toggleHealthCheck', disabled: true }
      ]
    });

    this.callParent();
  }
});
